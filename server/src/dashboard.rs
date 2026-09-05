//! Phase 3: Streamer Dashboard — สถิติ, ประวัติโดเนต, export CSV, ซ่อนข้อความ, อัปโหลดเสียง

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::auth::AuthUser;
use crate::AppState;

const MAX_SOUND_BYTES: usize = 2 * 1024 * 1024; // 2MB
const ALLOWED_SOUND_MIME: &[&str] = &["audio/mpeg", "audio/wav", "audio/x-wav", "audio/ogg"];

// ---------- Stats ----------

#[derive(Serialize)]
pub struct Stats {
    pub total_all: i64,
    pub total_today: i64,
    pub total_month: i64,
    pub count_all: i64,
    pub goal_amount: i64,
    pub series_30d: Vec<DayPoint>,
}

#[derive(Serialize)]
pub struct DayPoint {
    pub day: String, // YYYY-MM-DD
    pub total: i64,
}

pub async fn stats(State(state): State<AppState>, user: AuthUser) -> Result<Json<Stats>, (StatusCode, String)> {
    let db = &state.db;

    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount),0) t, COUNT(*) c FROM donations WHERE user_id = ? AND status='paid'",
    )
    .bind(&user.user_id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    let (total_all, count_all) = (row.get::<i64, _>("t"), row.get::<i64, _>("c"));

    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount),0) t FROM donations
         WHERE user_id=? AND status='paid' AND created_at >= (CAST(strftime('%s','now','localtime','start of day') AS INTEGER) * 1000)",
    )
    .bind(&user.user_id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    let total_today = row.get::<i64, _>("t");

    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount),0) t FROM donations
         WHERE user_id=? AND status='paid' AND created_at >= (CAST(strftime('%s','now','localtime','start of month') AS INTEGER) * 1000)",
    )
    .bind(&user.user_id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    let total_month = row.get::<i64, _>("t");

    // series 30 วัน — สร้างวันครบทุกวันด้วย recursive CTE (วันไหนไม่มีโดเนต = 0)
    let rows = sqlx::query(
        "WITH RECURSIVE days(d) AS (
            SELECT date('now','localtime','-29 days')
            UNION ALL SELECT date(d,'+1 day') FROM days WHERE d < date('now','localtime')
         )
         SELECT days.d, COALESCE(SUM(dn.amount),0) t
         FROM days
         LEFT JOIN donations dn
           ON date(dn.created_at/1000, 'unixepoch', 'localtime') = days.d
          AND dn.user_id = ? AND dn.status = 'paid'
         GROUP BY days.d ORDER BY days.d",
    )
    .bind(&user.user_id)
    .fetch_all(db)
    .await
    .map_err(db_err)?;

    let series_30d = rows
        .into_iter()
        .map(|r| DayPoint { day: r.get("d"), total: r.get("t") })
        .collect();

    let goal_amount = sqlx::query("SELECT goal_amount FROM settings WHERE user_id = ?")
        .bind(&user.user_id)
        .fetch_optional(db)
        .await
        .map_err(db_err)?
        .and_then(|r| r.try_get::<i64, _>("goal_amount").ok())
        .unwrap_or(0);

    Ok(Json(Stats { total_all, total_today, total_month, count_all, goal_amount, series_30d }))
}

fn db_err(e: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}"))
}

// ---------- Donation history ----------

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
    /// กรองช่วงวันที่ (epoch millis)
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub to: Option<i64>,
}

#[derive(Serialize)]
pub struct DonationItem {
    pub id: String,
    pub donor_name: String,
    pub amount: i64,
    pub message: String,
    pub sound: String,
    pub hidden: bool,
    pub created_at: i64,
    pub paid_at: Option<i64>,
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub items: Vec<DonationItem>,
    pub page: i64,
    pub total_pages: i64,
}

pub async fn history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, (StatusCode, String)> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);

    let pattern = format!("%{}%", q.q.unwrap_or_default().replace('%', ""));
    let from = q.from.unwrap_or(0);
    let to = q.to.unwrap_or(i64::MAX);

    let total: i64 = sqlx::query(
        "SELECT COUNT(*) c FROM donations
         WHERE user_id=? AND status='paid' AND created_at >= ? AND created_at <= ?
         AND (donor_name LIKE ? OR message LIKE ?)",
    )
    .bind(&user.user_id)
    .bind(from)
    .bind(to)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_one(&state.db)
    .await
    .map_err(db_err)?
    .get("c");

    let rows = sqlx::query(
        "SELECT id, donor_name, amount, message, sound, hidden, created_at, paid_at FROM donations
         WHERE user_id=? AND status='paid' AND created_at >= ? AND created_at <= ?
         AND (donor_name LIKE ? OR message LIKE ?)
         ORDER BY COALESCE(paid_at, created_at) DESC LIMIT ? OFFSET ?",
    )
    .bind(&user.user_id)
    .bind(from)
    .bind(to)
    .bind(&pattern)
    .bind(&pattern)
    .bind(per_page)
    .bind((page - 1) * per_page)
    .fetch_all(&state.db)
    .await
    .map_err(db_err)?;

    Ok(Json(HistoryResponse {
        items: rows
            .into_iter()
            .map(|r| DonationItem {
                id: r.get("id"),
                donor_name: r.get("donor_name"),
                amount: r.get("amount"),
                message: r.get("message"),
                sound: r.get("sound"),
                hidden: r.get::<i64, _>("hidden") != 0,
                created_at: r.get("created_at"),
                paid_at: r.get("paid_at"),
            })
            .collect(),
        page,
        total_pages: (total + per_page - 1) / per_page,
    }))
}

/// PATCH /api/me/donations/:id — ซ่อน/เปิดข้อความ (ownership ตรวจใน WHERE)
#[derive(Deserialize)]
pub struct PatchDonation {
    pub hidden: bool,
}

pub async fn patch_donation(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<PatchDonation>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let res = sqlx::query("UPDATE donations SET hidden = ? WHERE id = ? AND user_id = ?")
        .bind(body.hidden as i64)
        .bind(&id)
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .map_err(db_err)?;
    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "ไม่พบรายการ".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// GET /api/me/donations.csv — export ทั้งหมด (ฝั่ง Rust ทำ CSV)
pub async fn export_csv(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT donor_name, amount, message, paid_at FROM donations WHERE user_id=? AND status='paid' ORDER BY paid_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(db_err)?;

    let mut csv = String::from("donor_name,amount,message,paid_at_iso\n");
    for r in rows {
        let name: String = r.get("donor_name");
        let msg: String = r.get("message");
        let paid: Option<i64> = r.get("paid_at");
        let iso = paid.map(fmt_iso).unwrap_or_default();
        csv.push_str(&format!(
            "\"{}\",{},\"{}\",{}\n",
            name.replace('"', "\"\""),
            r.get::<i64, _>("amount"),
            msg.replace('"', "\"\""),
            iso
        ));
    }

    Ok((
        [(header::CONTENT_TYPE, "text/csv; charset=utf-8"),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"donations.csv\"")],
        csv,
    ))
}

fn fmt_iso(epoch_millis: i64) -> String {
    // ISO แบบง่ายจาก epoch (UTC) — เพียงพอสำหรับ export
    let secs = epoch_millis / 1000;
    let days = secs / 86400;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // วันที่จาก epoch (อัลกอรึทึม Cival)
    let mut days = days as i64;
    let mut year = 1970i64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if days < len { break; }
        days -= len;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_lens = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for &len in &month_lens {
        if days < len { break; }
        days -= len;
        month += 1;
    }
    format!("{year:04}-{month:02}-{days:02}T{h:02}:{m:02}:{s:02}Z")
}

// ---------- Wallet & Withdraw (mock — โอนจริงรอ Phase Omise Payout) ----------

#[derive(Serialize)]
pub struct Wallet {
    pub total_earned: i64,
    pub pending_withdraw: i64,
    pub total_withdrawn: i64,
    pub balance: i64,
}

#[derive(Serialize)]
pub struct Withdrawal {
    pub id: String,
    pub amount: i64,
    pub bank_name: String,
    pub bank_account: String,
    pub status: String,
    pub created_at: i64,
    pub paid_at: Option<i64>,
}

/// GET /api/me/wallet
pub async fn wallet(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    let total_earned = total_earned(&state.db, &user.user_id).await?;
    let (pending, withdrawn) = pending_and_withdrawn(&state.db, &user.user_id).await?;

    Ok(Json(Wallet {
        total_earned,
        pending_withdraw: pending,
        total_withdrawn: withdrawn,
        balance: total_earned - pending - withdrawn,
    }))
}

async fn total_earned(db: &SqlitePool, user_id: &str) -> Result<i64, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount),0) t FROM donations WHERE user_id=? AND status='paid'",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    Ok(row.get::<i64, _>("t"))
}

async fn pending_and_withdrawn(
    db: &SqlitePool,
    user_id: &str,
) -> Result<(i64, i64), (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT
           COALESCE(SUM(CASE WHEN status='pending' THEN amount END),0) p,
           COALESCE(SUM(CASE WHEN status='completed' THEN amount END),0) w
         FROM withdrawals WHERE user_id=? AND status IN ('pending','completed')",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    Ok((row.get("p"), row.get("w")))
}

async fn compute_balance(db: &SqlitePool, user_id: &str) -> Result<i64, (StatusCode, String)> {
    let total_earned = total_earned(db, user_id).await?;
    let (pending, withdrawn) = pending_and_withdrawn(db, user_id).await?;
    Ok(total_earned - pending - withdrawn)
}

/// GET /api/me/withdrawals
pub async fn list_withdrawals(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Withdrawal>>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT id, amount, bank_name, bank_account, status, created_at, paid_at
         FROM withdrawals WHERE user_id=? ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(db_err)?;

    Ok(Json(rows
        .into_iter()
        .map(|r| Withdrawal {
            id: r.get("id"),
            amount: r.get("amount"),
            bank_name: r.get("bank_name"),
            bank_account: r.get("bank_account"),
            status: r.get("status"),
            created_at: r.get("created_at"),
            paid_at: r.get("paid_at"),
        })
        .collect()))
}

#[derive(Deserialize)]
pub struct WithdrawInput {
    pub amount: i64,
    pub bank_name: String,
    pub bank_account: String,
}

/// POST /api/me/withdrawals — สร้างคำขอถอน (mock: สถานะ pending)
pub async fn create_withdrawal(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<WithdrawInput>,
) -> Result<(StatusCode, Json<Withdrawal>), (StatusCode, String)> {
    let bank_name = crate::sanitize::clean_text(&body.bank_name);
    let bank_account = body.bank_account.trim();
    if bank_name.is_empty() || bank_name.chars().count() > 60 {
        return Err((StatusCode::BAD_REQUEST, "กรุณาระบุชื่อธนาคาร".into()));
    }
    if bank_account.is_empty() || bank_account.len() > 30 || !bank_account.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return Err((StatusCode::BAD_REQUEST, "เลขบัญชีต้องเป็นตัวเลข 10-30 หลัก".into()));
    }
    if !(100..=1_000_000).contains(&body.amount) {
        return Err((StatusCode::BAD_REQUEST, "ถอนขั้นต่ำ ฿100 สูงสุด ฿1,000,000".into()));
    }

    // เช็คยอดคงเหลือ (earned − pending − completed)
    let balance = compute_balance(&state.db, &user.user_id).await?;
    if body.amount > balance {
        return Err((StatusCode::BAD_REQUEST, format!("ยอดคงเหลือไม่พอ (ถอนได้สูงสุด ฿{balance})")));
    }

    let id = crate::random_id();
    let now = crate::now_millis();
    sqlx::query(
        "INSERT INTO withdrawals (id, user_id, amount, bank_name, bank_account, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&id)
    .bind(&user.user_id)
    .bind(body.amount)
    .bind(&bank_name)
    .bind(bank_account)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(db_err)?;

    Ok((
        StatusCode::CREATED,
        Json(Withdrawal {
            id,
            amount: body.amount,
            bank_name,
            bank_account: bank_account.to_string(),
            status: "pending".into(),
            created_at: now,
            paid_at: None,
        }),
    ))
}

// ---------- Alert sound upload ----------

/// POST /api/me/alert-sound — multipart ฟิลด์ "file" (mp3/wav/ogg ≤ 2MB)
pub async fn upload_sound(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut saved_url = String::new();
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, format!("multipart: {e}")))? {
        if field.name() != Some("file") {
            continue;
        }
        let mime = field.content_type().unwrap_or("").to_string();
        if !ALLOWED_SOUND_MIME.contains(&mime.as_str()) {
            return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, format!("ชนิดไฟล์ไม่รองรับ: {mime}")));
        }
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("อ่านไฟล์ไม่สำเร็จ: {e}")))?;
        if data.len() > MAX_SOUND_BYTES {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "ไฟล์ใหญ่เกิน 2MB".into()));
        }

        let ext = match mime.as_str() {
            "audio/mpeg" => "mp3",
            "audio/ogg" => "ogg",
            _ => "wav",
        };
        let filename = format!("{}_{}.{}", user.user_id, crate::random_id()[..8].to_string(), ext);
        let dir = std::path::Path::new("uploads");
        tokio::fs::create_dir_all(dir).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("fs: {e}")))?;
        tokio::fs::write(dir.join(&filename), &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("fs: {e}")))?;
        saved_url = format!("/uploads/{filename}");
    }

    if saved_url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "ไม่พบฟิลด์ file".into()));
    }

    sqlx::query("UPDATE settings SET alert_sound_url = ? WHERE user_id = ?")
        .bind(&saved_url)
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .map_err(db_err)?;

    Ok(Json(serde_json::json!({ "url": saved_url })))
}
