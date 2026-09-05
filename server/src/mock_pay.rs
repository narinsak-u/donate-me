//! Mock payment provider: สร้าง QR ปลอม + หน้า "ธนาคารจำลอง" ให้กดยืนยันการจ่ายเงิน
//! เมื่อสลับเป็น Omise จริง ให้ลบ module นี้และเปลี่ยนเป็น OmiseProvider

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use hmac::{Hmac, Mac};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mock/qr/{ref}", get(mock_qr))
        .route("/mock/pay/{ref}", get(mock_pay_page))
        .route("/mock/webhook", post(crate::mock_webhook))
}

/// สร้าง QR จริงจาก payload ปลอม `promptpay-mock://...` (สร้างเอง ไม่พึ่งเน็ต)
async fn mock_qr(Path(payment_ref): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let ref_clean: String = payment_ref.trim_end_matches(".png").to_string();
    let is_valid = ref_clean.starts_with("mock_")
        && ref_clean.len() == 37
        && ref_clean[5..].bytes().all(|b| b.is_ascii_hexdigit());
    if !is_valid {
        return Err(StatusCode::NOT_FOUND);
    }

    let code = qrcode::QrCode::new(format!("promptpay-mock://{ref_clean}"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(256, 256)
        .dark_color(image::Luma([20u8]))
        .light_color(image::Luma([255u8]))
        .build();

    let mut png = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        [(header::CONTENT_TYPE, "image/png")],
        png.into_inner(),
    ))
}

/// หน้า "ธนาคารจำลอง" — ปุ่มกดยืนยันแล้วยิง webhook ที่ /mock/webhook พร้อม signature ที่ฝังมาจาก server
async fn mock_pay_page(
    State(state): State<AppState>,
    Path(payment_ref): Path<String>,
) -> Html<String> {
    let sig_success = sign(&state.webhook_secret, &format!("{payment_ref}|success"));
    let sig_failed = sign(&state.webhook_secret, &format!("{payment_ref}|failed"));

    Html(format!(r#"<!DOCTYPE html>
<html lang="th">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PromptPay Mock — จำลองการชำระเงิน</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: 'Segoe UI', 'Noto Sans Thai', sans-serif; }}
  body {{ min-height: 100vh; background: linear-gradient(135deg, #0b2b5e, #10429a); display: flex;
         align-items: center; justify-content: center; padding: 20px; }}
  .phone {{ background: #fff; border-radius: 24px; padding: 36px 32px; width: 100%; max-width: 380px;
           box-shadow: 0 25px 60px rgba(0,0,0,.35); text-align: center; }}
  .bank {{ font-size: 15px; color: #666; letter-spacing: 1px; }}
  h1 {{ font-size: 20px; color: #0b2b5e; margin: 6px 0 22px; }}
  img {{ width: 220px; height: 220px; image-rendering: pixelated; }}
  .ref {{ font-size: 12px; color: #999; margin: 12px 0 24px; word-break: break-all; }}
  .btn {{ width: 100%; padding: 15px; border: none; border-radius: 12px; font-size: 16px;
         font-weight: 700; cursor: pointer; margin-bottom: 10px; transition: opacity .15s; }}
  .btn:hover {{ opacity: .9; }}
  .ok {{ background: #22c55e; color: #fff; }}
  .fail {{ background: #f1f5f9; color: #64748b; }}
  #result {{ margin-top: 16px; font-weight: 700; min-height: 24px; }}
  .badge {{ background: #fef3c7; color: #92400e; font-size: 12px; padding: 4px 12px;
           border-radius: 999px; display: inline-block; margin-bottom: 16px; }}
</style>
</head>
<body>
<div class="phone">
  <div class="bank">MOCK BANK</div>
  <h1>PromptPay (จำลอง)</h1>
  <span class="badge"> sandbox — ไม่มีการตัดเงินจริง</span>
  <img src="/mock/qr/{payment_ref}.png" alt="QR mock">
  <div class="ref">ref: {payment_ref}</div>
  <button class="btn ok"   id="ok">✅ จำลองว่าจ่ายสำเร็จ</button>
  <button class="btn fail" id="no">❌ จำลองว่าจ่ายไม่สำเร็จ</button>
  <div id="result"></div>
</div>
<script>
async function pay(result, sig) {{
  document.getElementById('result').textContent = 'กำลังดำเนินการ...';
  const res = await fetch('/mock/webhook', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'application/json' }},
    body: JSON.stringify({{ payment_ref: '{payment_ref}', result, signature: sig }})
  }});
  document.getElementById('result').textContent =
    res.ok ? (result === 'success' ? '✅ สำเร็จ! ดู overlay เด้งได้เลย' : 'บันทึกว่าล้มเหลวแล้ว')
           : '❌ ผิดพลาด: ' + (await res.text());
}}
document.getElementById('ok').onclick  = () => pay('success', '{sig_success}');
document.getElementById('no').onclick  = () => pay('failed',  '{sig_failed}');
</script>
</body>
</html>"#))
}

fn sign(secret: &str, payload: &str) -> String {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
