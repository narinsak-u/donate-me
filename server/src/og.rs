//! OG tags สำหรับ social crawler (Discord/Twitter/FB ไม่รัน JS)
//! - `og_card_png()` สร้างรูป preview 1200×630 (gradient + หัวใจ) ครั้งเดียวในหน่วยความจำ
//! - `render_index()` อ่าน index.html แล้วฉีด og/twitter meta ตาม streamer ที่ขอมา

use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::IntoResponse;
use sqlx::Row;
use std::sync::OnceLock;

use crate::AppState;

static OG_PNG: OnceLock<Vec<u8>> = OnceLock::new();

/// รูป 1200×630: ไล่สี rose→violet + หัวใจกลางภาพ (วาดเอง ไม่ใช้ฟอนต์)
pub fn og_card_png() -> &'static [u8] {
    OG_PNG.get_or_init(|| {
        let (w, h) = (1200u32, 630u32);
        let mut img = image::RgbImage::new(w, h);

        // gradient ทแยง rose (#f43f5e) → violet (#7873f5)
        for (x, y, px) in img.enumerate_pixels_mut() {
            let t = (x as f32 / w as f32 + y as f32 / h as f32) / 2.0;
            let mix = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t) as u8 };
            *px = image::Rgb([mix(244, 120), mix(63, 115), mix(94, 245)]);
        }

        // หัวใจ = วงกลม 2 ลูก + สามเหลี่ยมชี้ลง
        let (cx, cy) = (w as i32 / 2, h as i32 / 2 - 40);
        let r = 150i32;
        let white = image::Rgb([255, 255, 255]);
        fill_circle(&mut img, cx - r / 2, cy, r / 2, white);
        fill_circle(&mut img, cx + r / 2, cy, r / 2, white);
        fill_triangle(&mut img, (cx - r, cy - 20), (cx + r, cy - 20), (cx, cy + r * 2), white);

        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode og png");
        png.into_inner()
    })
}

fn fill_circle(img: &mut image::RgbImage, cx: i32, cy: i32, r: i32, color: image::Rgb<u8>) {
    for y in (cy - r).max(0)..(cy + r).min(img.height() as i32) {
        for x in (cx - r).max(0)..(cx + r).min(img.width() as i32) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

/// จุดในสามเหลี่ยม (ใช้ sign ของ cross product)
fn in_triangle(px: i32, py: i32, a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let sign = |(x1, y1): (i32, i32), (x2, y2): (i32, i32)| {
        (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
    };
    let d1 = sign(a, b);
    let d2 = sign(b, c);
    let d3 = sign(c, a);
    let has_neg = d1 < 0 || d2 < 0 || d3 < 0;
    let has_pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(has_neg && has_pos)
}

fn fill_triangle(
    img: &mut image::RgbImage,
    a: (i32, i32),
    b: (i32, i32),
    c: (i32, i32),
    color: image::Rgb<u8>,
) {
    let min_x = a.0.min(b.0).min(c.0).max(0);
    let max_x = a.0.max(b.0).max(c.0).min(img.width() as i32);
    let min_y = a.1.min(b.1).min(c.1).max(0);
    let max_y = a.1.max(b.1).max(c.1).min(img.height() as i32);
    for y in min_y..max_y {
        for x in min_x..max_x {
            if in_triangle(x, y, a, b, c) {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// คืน index.html พร้อม OG meta ตาม ?u= (ไม่เจอ user → generic)
pub fn render_index(
    raw_html: &str,
    username: Option<&str>,
    display_name: Option<&str>,
    scheme: &str,
    host: &str,
    self_url: &str,
) -> String {
    let image_url = format!("{scheme}://{host}/og/og-card.png");
    let (title, desc) = match (username, display_name) {
        (Some(u), Some(name)) => (
            format!("สนับสนุน {} (@{}) ❤️ Donate Me", escape(name), escape(u)),
            "ส่งกำลังใจถึงสตรีมเมอร์คนโปรดผ่าน PromptPay — ป็อบอัพ + เสียงเด้งสด ๆ บนหน้าจอ stream ทันที 💜".to_string(),
        ),
        _ => (
            "Donate Me — สนับสนุนสตรีมเมอร์ ❤️".to_string(),
            "ระบบรับโดนตสำหรับสตรีมเมอร์: โดเนตผ่าน PromptPay → ป็อบอัพ + เสียงเด้งบน OBS แบบเรียลไทม์ 💜".to_string(),
        ),
    };
    let metas = format!(
        "<meta property=\"og:title\" content=\"{title}\">\n\
         <meta property=\"og:description\" content=\"{desc}\">\n\
         <meta property=\"og:image\" content=\"{image_url}\">\n\
         <meta property=\"og:type\" content=\"website\">\n\
         <meta property=\"og:url\" content=\"{self_url}\">\n\
         <meta name=\"twitter:card\" content=\"summary_large_image\">\n\
         <meta name=\"twitter:title\" content=\"{title}\">\n\
         <meta name=\"twitter:description\" content=\"{desc}\">\n\
         <meta name=\"twitter:image\" content=\"{image_url}\">"
    );
    // ฉีดหลัง </title> — แน่นอนกว่า placeholder comment (Vite build อาจ strip comment ทิ้ง)
    raw_html.replace("</title>", &format!("</title>\n{metas}"))
}

/// งานร่วมของ spa_index / spa_index_path — อ่าน user จาก DB แล้วเรนเดอร์ index พร้อม OG
async fn spa_with_username(
    state: AppState,
    headers: HeaderMap,
    username: Option<String>,
) -> impl IntoResponse {
    let html = std::fs::read_to_string("../web/dist/index.html").unwrap_or_else(|_| {
        "<!DOCTYPE html><html><head><title>Donate Me</title></head><body>index.html ไม่พบ — build web/ ก่อน (bun run build)</body></html>".into()
    });

    // ดึง display_name จาก DB (username: a-z0-9_ เท่านั้น กัน injection ตอนค้น)
    let mut display_name = None;
    let mut real_username = None;
    if let Some(u) = &username {
        if !u.is_empty()
            && u.len() <= 20
            && u.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            if let Ok(Some(row)) = sqlx::query("SELECT username, display_name FROM users WHERE username = ?")
                .bind(u)
                .fetch_optional(&state.db)
                .await
            {
                real_username = Some(row.get::<String, _>("username"));
                display_name = Some(row.get::<String, _>("display_name"));
            }
        }
    }

    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3000");
    let self_url = match &real_username {
        Some(u) => format!("{scheme}://{host}/u/{u}"),
        None => format!("{scheme}://{host}/"),
    };

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        render_index(&html, real_username.as_deref(), display_name.as_deref(), scheme, host, &self_url),
    )
}

/// handler สำหรับ GET / — ให้บริการ SPA พร้อม OG tags ตาม ?u= (ลิงก์แชร์รูปแบบเก่า)
pub async fn spa_index(
    State(state): State<AppState>,
    headers: HeaderMap,
    raw: axum::extract::RawQuery,
) -> impl IntoResponse {
    let query = raw.0.unwrap_or_default();
    let username = query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == "u").then(|| v.to_string())
    });
    spa_with_username(state, headers, username).await
}

/// handler สำหรับ GET /u/:username — หน้าโดเนตรูปแบบ path พร้อม OG tags
pub async fn spa_index_path(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> impl IntoResponse {
    spa_with_username(state, headers, Some(username)).await
}

pub fn og_png_response() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=86400")],
        og_card_png(),
    )
}
