mod auth;
mod dashboard;
mod mock_pay;
mod payment;
mod rate_limit;
mod sanitize;
mod users;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::StreamExt as _;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

// ---------- AppState ----------

pub type DonationTx = broadcast::Sender<Arc<DonationEvent>>;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tx: DonationTx,
    pub webhook_secret: String,
    pub jwt_secret: String,
    pub user_id: String, // Phase 1: streamer คนเดียว (hardcode "default")
    // Phase 4: rate limit แยกตามชนิดของ endpoint
    pub donate_limiter: Arc<rate_limit::RateLimiter>,
    pub auth_limiter: Arc<rate_limit::RateLimiter>,
    // Phase 6: ผู้ให้บริการชำระเงิน (mock หรือ Omise)
    pub payment: Arc<payment::PaymentProvider>,
}

#[derive(Serialize, Clone, Debug)]
pub struct DonationEvent {
    pub user_id: String,
    pub id: String,
    pub donor_name: String,
    pub amount: i64,
    pub message: String,
    pub sound: String,
}

// ---------- Models ----------

pub struct DonationRow {
    pub user_id: String,
    pub id: String,
    pub donor_name: String,
    pub amount: i64,
    pub message: String,
    pub sound: String,
    pub status: String,
    pub payment_ref: String,
}

impl From<&DonationRow> for DonationEvent {
    fn from(d: &DonationRow) -> Self {
        Self {
            user_id: d.user_id.clone(),
            id: d.id.clone(),
            donor_name: d.donor_name.clone(),
            amount: d.amount,
            message: d.message.clone(),
            sound: d.sound.clone(),
        }
    }
}

// ---------- API types ----------

#[derive(Deserialize)]
struct CreateDonation {
    name: String,
    amount: i64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    sound: String,
    /// สตรีมเมอร์ปลายทาง (username) — ไม่ส่ง = "default" (Phase 1 compat)
    #[serde(default)]
    username: Option<String>,
}

#[derive(Serialize)]
struct DonationCreated {
    id: String,
    qr_url: String,
    pay_url: String,
}

#[derive(Serialize)]
struct DonationStatus {
    id: String,
    status: String,
    amount: i64,
}

#[derive(Deserialize)]
struct MockWebhook {
    payment_ref: String,
    result: String, // "success" | "failed"
    signature: String,
}

// ---------- Handlers ----------

async fn healthz() -> &'static str {
    "ok"
}

async fn create_donation(
    State(state): State<AppState>,
    Json(body): Json<CreateDonation>,
) -> Result<(StatusCode, Json<DonationCreated>), (StatusCode, String)> {
    let name = sanitize::clean_text(&body.name);
    let message = sanitize::clean_text(&body.message);
    let sound = match body.sound.as_str() {
        "chime" | "coin" | "fanfare" | "tts" => body.sound.as_str(),
        _ => "chime",
    };

    if name.is_empty() || name.chars().count() > 50 {
        return Err((StatusCode::BAD_REQUEST, "ชื่อต้องยาว 1-50 ตัวอักษร".into()));
    }
    if !(1..=100_000).contains(&body.amount) {
        return Err((
            StatusCode::BAD_REQUEST,
            "จำนวนเงินต้องอยู่ระหว่าง 1-100,000 บาท".into(),
        ));
    }
    if message.chars().count() > 200 {
        return Err((StatusCode::BAD_REQUEST, "ข้อความยาวเกิน 200 ตัวอักษร".into()));
    }
    // Phase 4: mask คำหยาบ (เก็บทั้งชื่อและข้อความ)
    let name = sanitize::mask_bad_words(&name);
    let message = sanitize::mask_bad_words(&message);

    let id = random_id();

    // Phase 6: สร้าง intent ชำระเงินกับ provider (mock = QR ปลอม, omise = PromptPay จริง)
    let intent = state
        .payment
        .create_intent(body.amount, &id)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("สร้างรายการชำระเงินไม่สำเร็จ: {e}")))?;

    let now = now_millis();

    // หา user_id จาก username ปลายทาง
    let target_user = match &body.username {
        Some(username) => {
            let row = sqlx::query("SELECT id FROM users WHERE username = ?")
                .bind(username.to_lowercase())
                .fetch_optional(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;
            row.map(|r| r.get::<String, _>("id"))
                .ok_or((StatusCode::NOT_FOUND, format!("ไม่พบสตรีมเมอร์: {username}")))?
        }
        None => state.user_id.clone(),
    };

    sqlx::query(
        "INSERT INTO donations (id, user_id, donor_name, amount, message, sound, status, payment_ref, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?)",
    )
    .bind(&id)
    .bind(&target_user)
    .bind(name)
    .bind(body.amount)
    .bind(message)
    .bind(sound)
    .bind(&intent.provider_ref)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    // pay_url มีเฉพาะ mock (หน้าจำลองธนาคาร) — ฝั่ง omise จะสแกน QR ผ่านแอปธนาคารจริง
    let pay_url = if state.payment.is_mock() {
        format!("/mock/pay/{}", intent.provider_ref)
    } else {
        String::new()
    };

    Ok((
        StatusCode::CREATED,
        Json(DonationCreated {
            qr_url: intent.qr_url,
            pay_url,
            id,
        }),
    ))
}

async fn get_donation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DonationStatus>, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT id, donor_name, amount, message, sound, status, payment_ref FROM donations WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;
    let row = row.ok_or((StatusCode::NOT_FOUND, "ไม่พบรายการโดเนต".into()))?;
    Ok(Json(DonationStatus {
        id: row.get("id"),
        status: row.get("status"),
        amount: row.get("amount"),
    }))
}

async fn mock_webhook(
    State(state): State<AppState>,
    Json(body): Json<MockWebhook>,
) -> Result<Json<HashMap<&'static str, bool>>, (StatusCode, String)> {
    // ตรวจ HMAC signature เหมือน webhook จริง — กันลืมตอนสลับไป Omise (signature ผิด = client error ไม่ retry)
    let payload = format!("{}|{}", body.payment_ref, body.result);
    if !verify_signature(&state.webhook_secret, &payload, &body.signature) {
        return Err((StatusCode::UNAUTHORIZED, "invalid signature".into()));
    }

    let new_status = if body.result == "success" { "paid" } else { "failed" };

    // ถ้า failed ชั่วคราว (เช่น DB busy) ให้ retry สูงสุด 5 รอบ ห่างกันทีละ 200ms
    let mut last_err = String::new();
    for attempt in 1..=5 {
        match payment::settle(&state.db, &state.tx, &body.payment_ref, new_status).await {
            Ok(()) => return Ok(Json(HashMap::from([("ok", true)]))),
            Err(e) => {
                tracing::warn!("webhook attempt {attempt}/5 ล้มเหลว: {e}");
                last_err = e;
                tokio::time::sleep(Duration::from_millis(200 * attempt as u64)).await;
            }
        }
    }
    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("webhook ล้มเหลวหลัง retry 5 รอบ: {last_err}")))
}

/// Phase 6: webhook จริงจาก Omise — ตรวจสอบด้วยการ re-fetch event จาก Omise API
/// (Omise ไม่ส่ง HMAC มากับ webhook การ re-fetch ด้วย secret key คือวิธีตรวจมาตรฐาน)
async fn omise_webhook(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<HashMap<&'static str, bool>>, (StatusCode, String)> {
    let ev: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid json: {e}")))?;
    let event_id = ev["id"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "missing event id".into()))?;

    // re-fetch event จาก Omise — เนื้อหาจาก API ถือเป็น trusted (เนื้อหาใน body เชื่อไม่ได้)
    let (charge_id, omise_status) = state
        .payment
        .verify_webhook(event_id)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("ตรวจสอบ webhook ไม่ผ่าน: {e}")))?;

    // แปลงสถานะ Omise → สถานะของเรา (อื่น ๆ เช่น pending/reversed ไม่ต้องทำอะไร)
    let new_status = match omise_status.as_str() {
        "successful" => "paid",
        "failed" => "failed",
        _ => return Ok(Json(HashMap::from([("ok", true)]))),
    };

    let mut last_err = String::new();
    for attempt in 1..=5 {
        match payment::settle(&state.db, &state.tx, &charge_id, new_status).await {
            Ok(()) => return Ok(Json(HashMap::from([("ok", true)]))),
            Err(e) => {
                tracing::warn!("omise webhook attempt {attempt}/5 ล้มเหลว: {e}");
                last_err = e;
                tokio::time::sleep(Duration::from_millis(200 * attempt as u64)).await;
            }
        }
    }
    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("webhook ล้มเหลวหลัง retry 5 รอบ: {last_err}")))
}

fn row_into_donation(row: &sqlx::sqlite::SqliteRow) -> DonationRow {
    DonationRow {
        user_id: row.get("user_id"),
        id: row.get("id"),
        donor_name: row.get("donor_name"),
        amount: row.get("amount"),
        message: row.get("message"),
        sound: row.get("sound"),
        status: row.get("status"),
        payment_ref: row.get("payment_ref"),
    }
}

async fn events(
    State(state): State<AppState>,
    user: auth::AuthUser, // token ผูกกับสตรีมเมอร์ — ฟังเฉพาะโดเนตของตัวเอง
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, tokio_stream::wrappers::errors::BroadcastStreamRecvError>>>
{
    let my_id = user.user_id.clone();
    let rx = state.tx.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |msg| {
            // ส่งเฉพาะ event ของสตรีมเมอร์คนนี้ (test event id="test" ส่งให้ทุกคน)
            let ok = match &msg {
                Ok(d) => d.user_id == my_id || d.id == "test",
                Err(_) => false,
            };
            msg.ok()
                .filter(|_| ok)
                .map(|d| Ok(Event::default().event("donation").data(serde_json::to_string(&*d).unwrap())))
        });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn test_alert(State(state): State<AppState>, user: auth::AuthUser) -> Json<HashMap<&'static str, bool>> {
    let _ = state.tx.send(Arc::new(DonationEvent {
        user_id: user.user_id,
        id: "test".into(),
        donor_name: "ทดสอบระบบ".into(),
        amount: 100,
        message: "นี่คือการทดสอบแจ้งเตือน".into(),
        sound: "chime".into(),
    }));
    Json(HashMap::from([("ok", true)]))
}

// ---------- utils ----------

fn random_id() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn verify_signature(secret: &str, payload: &str, sig_hex: &str) -> bool {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let Ok(mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    let mut mac = mac;
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    // constant-time compare
    if expected.len() != sig_hex.len() {
        return false;
    }
    expected
        .bytes()
        .zip(sig_hex.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

// ---------- main ----------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,donate_me_api=debug".into()),
        )
        .init();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://donations.db?mode=rwc".into());
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("ไม่สามารถเชื่อมต่อฐานข้อมูลได้");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("migration ล้มเหลว");

    let (tx, _rx) = broadcast::channel(64);
    let state = AppState {
        db,
        tx,
        webhook_secret: std::env::var("MOCK_WEBHOOK_SECRET")
            .unwrap_or_else(|_| "mock-secret-change-me".into()),
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "jwt-dev-secret-change-me".into()),
        user_id: std::env::var("DEFAULT_USER_ID").unwrap_or_else(|_| "default".into()),
        donate_limiter: Arc::new(rate_limit::RateLimiter::default()),
        auth_limiter: Arc::new(rate_limit::RateLimiter::default()),
        payment: Arc::new(payment::PaymentProvider::from_env()),
    };
    let is_mock = state.payment.is_mock();

    // build ฝั่ง web ก่อน แล้ว Axum serve ทั้ง SPA และ overlay จาก web/dist
    let web_dist = std::path::Path::new("../web/dist");
    // mock routes เฉพาะตอนยังใช้ MockProvider — ตอนใช้ Omise จริงปิดทิ้ง (กันยิง mock webhook)
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/api/donate",
            post(create_donation).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                rate_limit::donate_limit,
            )),
        )
        .route("/api/donate/{id}", get(get_donation))
        .route("/api/test-alert", post(test_alert))
        .route("/webhooks/omise", post(omise_webhook))
        .route(
            "/api/auth/register",
            post(auth::register).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                rate_limit::auth_limit,
            )),
        )
        .route(
            "/api/auth/login",
            post(auth::login).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                rate_limit::auth_limit,
            )),
        )
        .route("/api/me", get(auth::me))
        .route("/api/u/{username}", get(users::public_profile))
        .route("/api/u/{username}/top", get(users::top_donators))
        .route("/api/me/settings", get(users::get_settings).put(users::update_settings))
        .route("/api/me/stats", get(dashboard::stats))
        .route("/api/me/donations", get(dashboard::history))
        .route("/api/me/donations.csv", get(dashboard::export_csv))
        .route("/api/me/donations/{id}", axum::routing::patch(dashboard::patch_donation))
        .route("/api/me/alert-sound", post(dashboard::upload_sound))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .route("/events", get(events))
        .merge(if is_mock { mock_pay::router() } else { Router::new() })
        .fallback_service(ServeDir::new(web_dist))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("พอร์ต 3000 ไม่ว่าง");
    tracing::info!("✅ Donate Me API รันที่ http://localhost:3000 (overlay: http://localhost:3000/overlay.html)");
    // ConnectInfo จำเป็นสำหรับ rate limiter ระบุ IP
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
