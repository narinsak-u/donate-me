//! Phase 2: ระบบสมาชิก — สมัคร/ล็อกอิน (argon2 + JWT) + AuthUser extractor

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{random_id, now_millis, AppState};

const RESERVED_NAMES: &[&str] = &[
    "admin", "api", "root", "me", "mock", "events", "donate", "overlay", "settings", "u",
];

// ---------- JWT ----------

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String, // user id
    exp: usize,  // unix seconds
}

fn jwt_secret(state: &AppState) -> &str {
    &state.jwt_secret
}

fn make_token(state: &AppState, user_id: &str) -> String {
    let exp = now_millis() / 1000 + 7 * 24 * 3600; // อายุ 7 วัน
    encode(
        &Header::default(),
        &Claims { sub: user_id.into(), exp: exp as usize },
        &EncodingKey::from_secret(jwt_secret(state).as_bytes()),
    )
    .unwrap()
}

// ---------- AuthUser extractor: ใส่ใน handler ไหนก็ได้เพื่อบังคับ login ----------

pub struct AuthUser {
    pub user_id: String,
}

/// อ่าน JWT จาก Authorization header หรือ ?token= (EventSource ใส่ header ไม่ได้)
fn extract_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
        .or_else(|| {
            let q = parts.uri.query()?;
            q.split('&').find_map(|kv| {
                let (k, v) = kv.split_once('=')?;
                (k == "token").then(|| urldecode(v))
            })
        })
}

fn decode_user_id(token: &str, state: &AppState) -> Option<String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret(state).as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|c| c.claims.sub)
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)
            .ok_or((StatusCode::UNAUTHORIZED, "ต้องล็อกอินก่อน (missing bearer token)"))?;

        let user_id = decode_user_id(&token, state)
            .ok_or((StatusCode::UNAUTHORIZED, "token ไม่ถูกต้องหรือหมดอายุ"))?;

        Ok(AuthUser { user_id })
    }
}

// ---------- MaybeAuthUser: เหมือน AuthUser แต่ไม่ reject ถ้าไม่มี token ----------

pub struct MaybeAuthUser {
    pub user_id: Option<String>,
}

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user_id = extract_token(parts).and_then(|t| decode_user_id(&t, state));
        Ok(MaybeAuthUser { user_id })
    }
}

fn urldecode(s: &str) -> String {
    // percent-decode แบบเบา ๆ เพียงพอกับ JWT (A-Za-z0-9 กับ -_.~ ไม่ต้อง encode)
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------- API types ----------

#[derive(Deserialize)]
pub struct RegisterInput {
    username: String,
    email: String,
    password: String,
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginInput {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: PublicUser,
}

#[derive(Serialize)]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
}

pub(crate) fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("hash error: {e}"))
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

fn validate_username(username: &str) -> Result<(), String> {
    let re = regex::Regex::new(r"^[a-z0-9_]{3,20}$").unwrap();
    if !re.is_match(username) {
        return Err("username ต้องเป็น a-z, 0-9, _ ยาว 3-20 ตัว".into());
    }
    if RESERVED_NAMES.contains(&username) {
        return Err("ชื่อนี้ถูกสงวนไว้".into());
    }
    Ok(())
}

// ---------- Handlers ----------

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterInput>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    let username = body.username.trim().to_lowercase();
    let email = body.email.trim().to_lowercase();

    validate_username(&username).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    if !email.contains('@') || email.len() > 254 {
        return Err((StatusCode::BAD_REQUEST, "อีเมลไม่ถูกต้อง".into()));
    }
    if body.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "รหัสผ่านต้องยาวอย่างน้อย 8 ตัวอักษร".into()));
    }

    let hash = hash_password(&body.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = random_id();
    let display_name = body.display_name.unwrap_or_else(|| username.clone());
    let exists = sqlx::query("SELECT 1 FROM users WHERE username = ? OR email = ?")
        .bind(&username)
        .bind(&email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;
    if exists.is_some() {
        return Err((StatusCode::CONFLICT, "username หรืออีเมลนี้ถูกใช้แล้ว".into()));
    }

    sqlx::query("INSERT INTO users (id, username, email, password_hash, display_name, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&username)
        .bind(&email)
        .bind(&hash)
        .bind(&display_name)
        .bind(now_millis())
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    // สร้าง settings row เริ่มต้นให้ทันที
    sqlx::query("INSERT INTO settings (user_id) VALUES (?)")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token: make_token(&state, &id),
            user: PublicUser { id, username, display_name },
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id, username, display_name, password_hash FROM users WHERE email = ?")
        .bind(body.email.trim().to_lowercase())
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;

    let row = row.ok_or((StatusCode::UNAUTHORIZED, "อีเมลหรือรหัสผ่านไม่ถูกต้อง".to_string()))?;
    let hash: String = row.get("password_hash");
    if !verify_password(&body.password, &hash) {
        return Err((StatusCode::UNAUTHORIZED, "อีเมลหรือรหัสผ่านไม่ถูกต้อง".into()));
    }

    let user = PublicUser {
        id: row.get("id"),
        username: row.get("username"),
        display_name: row.get("display_name"),
    };
    Ok(Json(AuthResponse { token: make_token(&state, &user.id), user }))
}

pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<PublicUser>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id, username, display_name FROM users WHERE id = ?")
        .bind(&user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {e}")))?;
    let row = row.ok_or((StatusCode::NOT_FOUND, "ไม่พบผู้ใช้".into()))?;
    Ok(Json(PublicUser {
        id: row.get("id"),
        username: row.get("username"),
        display_name: row.get("display_name"),
    }))
}
