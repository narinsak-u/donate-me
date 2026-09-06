//! Phase 7: ยืนยันสลิป — ผู้ชมโอนเข้าบัญชีสตรีมเมอร์ตรง แล้วแนบสลิป
//! สตรีมเมอร์อนุมัติ/ปฏิเสธใน "คิวสลิป" ของ dashboard — อนุมัติแล้ว alert ขึ้นจอทันที
//!
//! ความปลอดภัย: สลิปคือข้อมูลส่วนบุคคล — เก็บในโฟลเดอร์ `slips/` นอก `/uploads`
//! (ไม่เสิร์ฟสาธารณะ) และเปิดดูได้เฉพาะเจ้าของบัญชีผ่าน /api/me/slips/:id/image

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sqlx::Row;

use crate::auth::AuthUser;
use crate::AppState;

const MAX_SLIP_BYTES: usize = 5 * 1024 * 1024; // 5MB
const SLIP_DIR: &str = "slips";

fn db_err(e: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}"))
}

/// ตรวจ magic bytes ของไฟล์ — เชื่อเนื้อไฟล์จริง ไม่เชื่อ MIME ที่ client บอกมา
fn sniff_image(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        Some("image/jpeg")
    } else {
        None
    }
}

// ---------- ฝั่งผู้ชม ----------

/// POST /api/donate/:id/slip — แนบสลิปหลังโอน (ไม่ต้อง login, rate limit ตาม donate_limit)
pub async fn upload_slip(
    State(state): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err((StatusCode::BAD_REQUEST, "รหัสรายการไม่ถูกต้อง".into()));
    }

    let mut saved: Option<(String, &'static str)> = None; // (filename, content_type)
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("multipart: {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("อ่านไฟล์ไม่สำเร็จ: {e}")))?;
        if data.len() > MAX_SLIP_BYTES {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "ไฟล์ใหญ่เกิน 5MB".into()));
        }
        let ctype = sniff_image(&data).ok_or((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "รับเฉพาะรูป PNG / JPEG เท่านั้น".to_string(),
        ))?;
        let filename = format!("{}.{}", crate::random_id(), if ctype == "image/png" { "png" } else { "jpg" });

        tokio::fs::create_dir_all(SLIP_DIR)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("fs: {e}")))?;
        tokio::fs::write(std::path::Path::new(SLIP_DIR).join(&filename), &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("fs: {e}")))?;
        saved = Some((filename, ctype));
    }
    let (filename, _) = saved.ok_or((StatusCode::BAD_REQUEST, "ไม่พบฟิลด์ file".into()))?;

    // รับเฉพาะรายการโอนตรงที่ยัง pending และไม่หมดอายุ — แนบซ้ำ/อัปเดตสถานะแล้ว = ปฏิเสธ
    let rows = sqlx::query(
        "UPDATE donations SET slip_path = ?, slip_at = ?, status = 'awaiting_review'
         WHERE id = ? AND payment_ref LIKE 'direct\\_%' ESCAPE '\\'
           AND status = 'pending' AND (expires_at IS NULL OR expires_at >= ?)",
    )
    .bind(&filename)
    .bind(crate::now_millis())
    .bind(&id)
    .bind(crate::now_millis())
    .execute(&state.db)
    .await
    .map_err(db_err)?;

    if rows.rows_affected() == 0 {
        // รายการไม่รับสลิปแล้ว — เผ่นไฟล์ที่เพิ่งเขียนทิ้ง
        let _ = tokio::fs::remove_file(std::path::Path::new(SLIP_DIR).join(&filename)).await;
        return Err((
            StatusCode::CONFLICT,
            "รายการนี้รับสลิปไม่ได้ (หมดอายุ, แนบแล้ว หรือไม่ใช่ช่องทางโอนตรง)".into(),
        ));
    }

    Ok(Json(serde_json::json!({ "ok": true, "status": "awaiting_review" })))
}

// ---------- ฝั่งสตรีมเมอร์ (คิวสลิป) ----------

#[derive(Serialize)]
pub struct SlipItem {
    pub id: String,
    pub donor_name: String,
    pub amount: i64,
    pub message: String,
    pub sound: String,
    pub status: String,
    pub has_image: bool,
    pub created_at: i64,
    pub slip_at: Option<i64>,
    pub reviewed_at: Option<i64>,
    pub review_note: String,
}

/// GET /api/me/slips — คิวรอตรวจขึ้นก่อน ตามด้วยรายการที่พิจารณาแล้วล่าสุด
pub async fn list_slips(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<SlipItem>>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT id, donor_name, amount, message, sound, status, slip_path, created_at, slip_at, reviewed_at, review_note
         FROM donations
         WHERE user_id = ? AND (status = 'awaiting_review' OR slip_path != '')
         ORDER BY CASE status WHEN 'awaiting_review' THEN 0 ELSE 1 END,
                  COALESCE(slip_at, created_at) DESC
         LIMIT 50",
    )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(db_err)?;

    Ok(Json(rows
        .into_iter()
        .map(|r| SlipItem {
            id: r.get("id"),
            donor_name: r.get("donor_name"),
            amount: r.get("amount"),
            message: r.get("message"),
            sound: r.get("sound"),
            status: r.get("status"),
            has_image: !r.get::<String, _>("slip_path").is_empty(),
            created_at: r.get("created_at"),
            slip_at: r.get("slip_at"),
            reviewed_at: r.get("reviewed_at"),
            review_note: r.get("review_note"),
        })
        .collect()))
}

/// GET /api/me/slips/:id/image — เปิดดูสลิป (เจ้าของเท่านั้น)
pub async fn slip_image(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let row = sqlx::query("SELECT slip_path FROM donations WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(db_err)?
        .ok_or((StatusCode::NOT_FOUND, "ไม่พบรายการ".into()))?;
    let path: String = row.get("slip_path");
    if path.is_empty() {
        return Err((StatusCode::NOT_FOUND, "รายการนี้ไม่มีรูปสลิป".into()));
    }

    let data = tokio::fs::read(std::path::Path::new(SLIP_DIR).join(&path))
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "ไฟล์สลิปหาย".into()))?;
    let ctype = sniff_image(&data).unwrap_or("application/octet-stream");
    Ok((
        [
            (header::CONTENT_TYPE, ctype),
            (header::CACHE_CONTROL, "private, max-age=60"),
        ],
        data,
    ))
}

/// POST /api/me/slips/:id/approve — อนุมัติ = ตั้ง paid + alert ขึ้นจอ (idempotent ผ่าน settle)
pub async fn approve_slip(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let row = sqlx::query("SELECT payment_ref FROM donations WHERE id = ? AND user_id = ? AND status = 'awaiting_review'")
        .bind(&id)
        .bind(&user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(db_err)?
        .ok_or((StatusCode::NOT_FOUND, "ไม่พบรายการที่รอตรวจ".into()))?;
    let payment_ref: String = row.get("payment_ref");

    crate::payment::settle(&state.db, &state.tx, &payment_ref, "paid")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    sqlx::query("UPDATE donations SET reviewed_at = ? WHERE id = ?")
        .bind(crate::now_millis())
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(db_err)?;

    Ok(Json(serde_json::json!({ "ok": true, "status": "paid" })))
}

#[derive(serde::Deserialize)]
pub struct RejectBody {
    #[serde(default)]
    pub note: String,
}

/// POST /api/me/slips/:id/reject — ปฏิเสธพร้อมเหตุผล (ผู้ชมเห็นบนหน้าจ่าย)
pub async fn reject_slip(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    body: Json<RejectBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let note = crate::sanitize::clean_text(&body.note);
    if note.chars().count() > 200 {
        return Err((StatusCode::BAD_REQUEST, "เหตุผลยาวเกิน 200 ตัวอักษร".into()));
    }
    let rows = sqlx::query(
        "UPDATE donations SET status = 'rejected', review_note = ?, reviewed_at = ?
         WHERE id = ? AND user_id = ? AND status = 'awaiting_review'",
    )
    .bind(&note)
    .bind(crate::now_millis())
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(db_err)?;
    if rows.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "ไม่พบรายการที่รอตรวจ".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true, "status": "rejected" })))
}

// ---------- ตัวอย่าง QR สำหรับ dashboard ----------

#[derive(serde::Deserialize)]
pub struct QrQuery {
    #[serde(default)]
    pub amount: Option<i64>,
}

/// GET /api/me/promptpay-qr?amount=20 — ตัวอย่าง QR จาก promptpay_id ของตัวเอง (ทดสอบก่อนใช้จริง)
pub async fn promptpay_qr(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<QrQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let amount = q.amount.unwrap_or(20).clamp(1, 100_000);
    let row = sqlx::query("SELECT promptpay_id FROM settings WHERE user_id = ?")
        .bind(&user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(db_err)?
        .ok_or((StatusCode::NOT_FOUND, "ไม่พบ settings".into()))?;
    let pp: String = row.get("promptpay_id");
    let normalized = crate::promptpay::normalize_id(&pp)
        .ok_or((StatusCode::BAD_REQUEST, "ยังไม่ได้ตั้งเบอร์พร้อมเพย์ (หรือรูปแบบไม่ถูกต้อง)".into()))?;

    let png = crate::promptpay::qr_png(&crate::promptpay::payload(&normalized, amount * 100))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(([(header::CONTENT_TYPE, "image/png")], png))
}
