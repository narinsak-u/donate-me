//! Phase 4: In-memory rate limiter (sliding window ต่อ IP)
//! เพียงพอสำหรับ instance เดียว — ถ้า scale หลาย server ค่อยเปลี่ยนเป็น Redis (ตาม PLAN.md)

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{extract::{ConnectInfo, Request, State}, http::StatusCode, middleware::Next, response::{IntoResponse, Response}};

use crate::AppState;

#[derive(Default)]
pub struct RateLimiter {
    /// ip -> timestamps ของ request ที่ยังอยู่ในหน้าต่าง
    hits: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    /// คืน true = ผ่าน; false = โดน limit
    fn check(&self, ip: IpAddr, max: usize, window: Duration) -> bool {
        let mut hits = self.hits.lock().unwrap();
        let now = Instant::now();
        let entry = hits.entry(ip).or_default();
        entry.retain(|t| now.duration_since(*t) < window);
        if entry.len() >= max {
            entry.push(now); // นับต่อเพื่อรายงานต่อเนื่อง
            return false;
        }
        entry.push(now);

        // เก็บกวาด IP ที่เงียบนาน เมื่อแผนที่โตเกิน
        if hits.len() > 10_000 {
            hits.retain(|_, v| v.iter().any(|t| now.duration_since(*t) < window));
        }
        true
    }
}

/// ฟอร์มโดเนต: 5 ครั้ง/นาที/IP
pub async fn donate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    run_limit(&state.donate_limiter, addr.ip(), 5, req, next).await
}

/// auth (login/register): 10 ครั้ง/นาที/IP — กัน brute force รหัสผ่าน
pub async fn auth_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    run_limit(&state.auth_limiter, addr.ip(), 10, req, next).await
}

async fn run_limit(limiter: &RateLimiter, ip: IpAddr, max: usize, req: Request, next: Next) -> Response {
    if !limiter.check(ip, max, Duration::from_secs(60)) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "พยายามบ่อยเกินไป กรุณารอสักครู่แล้วลองใหม่",
        )
            .into_response();
    }
    next.run(req).await
}
