//! Phase 2: โปรไฟล์สาธารณะ + การตั้งค่าของสตรีมเมอร์

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::auth::AuthUser;
use crate::AppState;

// ---------- Public profile ----------

#[derive(Serialize)]
pub struct PublicProfile {
    pub username: String,
    pub display_name: String,
    pub goal_amount: i64,
    pub theme: String,
    pub show_leaderboard: bool,
}

/// GET /api/u/:username — ข้อมูลที่หน้าโดเนตสาธารณะต้องใช้
pub async fn public_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<PublicProfile>, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT u.username, u.display_name, s.goal_amount, s.theme, s.show_leaderboard
         FROM users u LEFT JOIN settings s ON s.user_id = u.id
         WHERE u.username = ?",
    )
    .bind(username.to_lowercase())
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    let row = row.ok_or((StatusCode::NOT_FOUND, "ไม่พบสตรีมเมอร์คนนี้".into()))?;
    Ok(Json(PublicProfile {
        username: row.get("username"),
        display_name: row.get("display_name"),
        goal_amount: row.get::<Option<i64>, _>("goal_amount").unwrap_or(0),
        theme: row.get::<Option<String>, _>("theme").unwrap_or_else(|| "pink".into()),
        show_leaderboard: row.get::<Option<i64>, _>("show_leaderboard").unwrap_or(1) != 0,
    }))
}

// ---------- Top donators (leaderboard) ----------

#[derive(Serialize)]
pub struct TopDonator {
    pub donor_name: String,
    pub total: i64,
    pub count: i64,
}

/// GET /api/u/:username/top — รวมยอดตามชื่อผู้โดเนต (top 10)
pub async fn top_donators(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<TopDonator>>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT d.donor_name, SUM(d.amount) total, COUNT(*) count
         FROM donations d
         JOIN users u ON u.id = d.user_id
         WHERE u.username = ? AND d.status = 'paid' AND d.hidden = 0
         GROUP BY d.donor_name
         ORDER BY total DESC
         LIMIT 10",
    )
    .bind(username.to_lowercase())
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    Ok(Json(rows
        .into_iter()
        .map(|r| TopDonator {
            donor_name: r.get("donor_name"),
            total: r.get("total"),
            count: r.get("count"),
        })
        .collect()))
}

// ---------- Recent live feed ----------

#[derive(Serialize)]
pub struct RecentDonation {
    pub donor_name: String,
    pub amount: i64,
    pub message: String,
    pub paid_at: i64,
}

/// GET /api/u/:username/recent — โดเนตล่าสุด 5 รายการ (ข้ามข้อความที่ซ่อน) สำหรับ live feed
pub async fn recent_donations(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<RecentDonation>>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT d.donor_name, d.amount, d.message, COALESCE(d.paid_at, d.created_at) t
         FROM donations d JOIN users u ON u.id = d.user_id
         WHERE u.username = ? AND d.status = 'paid' AND d.hidden = 0
         ORDER BY t DESC LIMIT 5",
    )
    .bind(username.to_lowercase())
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    Ok(Json(rows
        .into_iter()
        .map(|r| RecentDonation {
            donor_name: r.get("donor_name"),
            amount: r.get("amount"),
            message: r.get("message"),
            paid_at: r.get("t"),
        })
        .collect()))
}

// ---------- Settings (ต้อง login เจ้าของเท่านั้น) ----------

#[derive(Serialize)]
pub struct Settings {
    pub theme: String,
    pub goal_amount: i64,
    pub alert_duration_sec: i64,
    pub tts_enabled: bool,
    pub alert_text: String,
    pub alert_sound_url: String,
    pub alert_image_url: String,
    pub show_leaderboard: bool,
}

#[derive(Deserialize)]
pub struct UpdateSettings {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub goal_amount: Option<i64>,
    #[serde(default)]
    pub alert_duration_sec: Option<i64>,
    #[serde(default)]
    pub tts_enabled: Option<bool>,
    #[serde(default)]
    pub alert_text: Option<String>,
    #[serde(default)]
    pub alert_image_url: Option<String>,
    #[serde(default)]
    pub show_leaderboard: Option<bool>,
}

pub async fn get_settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Settings>, (StatusCode, String)> {
    let row = fetch_settings(&state, &user.user_id).await?;
    Ok(Json(row))
}

pub async fn update_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateSettings>,
) -> Result<Json<Settings>, (StatusCode, String)> {
    // อ่านค่าปัจจุบันก่อน แล้ว merge (PATCH semantics)
    let cur = fetch_settings(&state, &user.user_id).await?;

    let theme = match body.theme.as_deref() {
        Some(t) if ["pink", "blue", "green", "dark"].contains(&t) => t.to_string(),
        Some(_) => return Err((StatusCode::BAD_REQUEST, "theme ต้องเป็น pink/blue/green/dark".into())),
        None => cur.theme,
    };
    let goal_amount = body.goal_amount.unwrap_or(cur.goal_amount).clamp(0, 10_000_000);
    let alert_duration_sec = body.alert_duration_sec.unwrap_or(cur.alert_duration_sec).clamp(3, 30);
    let tts_enabled = body.tts_enabled.unwrap_or(cur.tts_enabled);
    let alert_text = body.alert_text.unwrap_or(cur.alert_text);
    if alert_text.chars().count() > 200 {
        return Err((StatusCode::BAD_REQUEST, "alert_text ยาวเกิน 200 ตัวอักษร".into()));
    }
    // media alert: ต้องเป็น URL https เท่านั้น (กัน javascript: / data: scheme)
    let alert_image_url = match body.alert_image_url.unwrap_or(cur.alert_image_url) {
        u if u.is_empty() => String::new(),
        u if u.starts_with("https://") => u,
        _ => return Err((StatusCode::BAD_REQUEST, "alert_image_url ต้องเป็น https:// เท่านั้น".into())),
    };
    let show_leaderboard = body.show_leaderboard.unwrap_or(cur.show_leaderboard);

    sqlx::query(
        "UPDATE settings SET theme = ?, goal_amount = ?, alert_duration_sec = ?, tts_enabled = ?, alert_text = ?, alert_image_url = ?, show_leaderboard = ? WHERE user_id = ?",
    )
    .bind(&theme)
    .bind(goal_amount)
    .bind(alert_duration_sec)
    .bind(tts_enabled as i64)
    .bind(&alert_text)
    .bind(&alert_image_url)
    .bind(show_leaderboard as i64)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    let updated = fetch_settings(&state, &user.user_id).await?;
    Ok(Json(updated))
}

async fn fetch_settings(state: &AppState, user_id: &str) -> Result<Settings, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT theme, goal_amount, alert_duration_sec, tts_enabled, alert_text, alert_sound_url, alert_image_url, show_leaderboard FROM settings WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;
    let row = row.ok_or((StatusCode::NOT_FOUND, "ไม่พบ settings".into()))?;
    Ok(Settings {
        theme: row.get("theme"),
        goal_amount: row.get("goal_amount"),
        alert_duration_sec: row.get("alert_duration_sec"),
        tts_enabled: row.get::<i64, _>("tts_enabled") != 0,
        alert_text: row.get("alert_text"),
        alert_sound_url: row.get("alert_sound_url"),
        alert_image_url: row.get("alert_image_url"),
        show_leaderboard: row.get::<i64, _>("show_leaderboard") != 0,
    })
}
