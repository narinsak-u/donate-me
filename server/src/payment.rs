//! Phase 6: PaymentProvider — เลือกผู้ให้บริการชำระเงินจาก env
//! - ตั้ง OMISE_SECRET_KEY → Omise จริง (PromptPay, test mode ก่อน)
//! - ไม่ตั้ง → MockProvider (Mock Bank + QR ปลอม) สำหรับ dev
//!
//! หมายเหตุความปลอดภัย: Omise webhook ไม่มี HMAC signature มาให้ตรวจ —
//! วิธีมาตรฐานคือ "re-fetch" event จาก Omise API ด้วย secret key ก่อนเชื่อถือเนื้อหา

use serde_json::Value;
use sqlx::SqlitePool;

const OMISE_API: &str = "https://api.omise.co";

pub struct PaymentIntent {
    pub provider_ref: String, // mock: mock_{id}, omise: chrg_...
    pub qr_url: String,       // ส่งให้ PayPage แสดง <img>
}

pub enum PaymentProvider {
    Mock,
    Omise { secret: String, client: reqwest::Client },
}

impl PaymentProvider {
    pub fn from_env() -> Self {
        match std::env::var("OMISE_SECRET_KEY") {
            Ok(key) if !key.is_empty() => {
                tracing::info!("💳 PaymentProvider = Omise (ระวัง: ถ้า key ขึ้นต้นด้วย skey_test คือ test mode)");
                Self::Omise { secret: key, client: reqwest::Client::new() }
            }
            _ => {
                tracing::info!("💳 PaymentProvider = Mock (ไม่มี OMISE_SECRET_KEY ใน .env)");
                Self::Mock
            }
        }
    }

    pub fn is_mock(&self) -> bool {
        matches!(self, Self::Mock)
    }

    /// สร้าง intent ชำระเงิน (amount เป็นบาท, คืน QR ให้ PayPage)
    pub async fn create_intent(&self, amount_thb: i64, donation_id: &str) -> Result<PaymentIntent, String> {
        match self {
            Self::Mock => Ok(PaymentIntent {
                provider_ref: format!("mock_{donation_id}"),
                qr_url: format!("/mock/qr/mock_{donation_id}.png"),
            }),
            Self::Omise { secret, client } => {
                let satang = (amount_thb * 100).to_string();

                // 1) สร้าง PromptPay source
                let resp = client
                    .post(format!("{OMISE_API}/sources"))
                    .basic_auth(secret, Some(""))
                    .form(&[("amount", satang.as_str()), ("currency", "thb"), ("type", "promptpay")])
                    .send()
                    .await
                    .map_err(|e| format!("omise source: {e}"))?;
                let source: Value = resp.json().await.map_err(|e| format!("omise source json: {e}"))?;
                if source["object"].as_str() != Some("source") {
                    return Err(format!("omise source error: {source}"));
                }
                let src_id = source["id"].as_str().ok_or("omise: missing source id")?;

                // 2) สร้าง charge จาก source (status เริ่มต้น = pending)
                let resp = client
                    .post(format!("{OMISE_API}/charges"))
                    .basic_auth(secret, Some(""))
                    .form(&[
                        ("amount", satang.as_str()),
                        ("currency", "thb"),
                        ("source", src_id),
                        ("metadata[donation_id]", donation_id),
                    ])
                    .send()
                    .await
                    .map_err(|e| format!("omise charge: {e}"))?;
                let charge: Value = resp.json().await.map_err(|e| format!("omise charge json: {e}"))?;
                if charge["object"].as_str() != Some("charge") {
                    return Err(format!("omise charge error: {charge}"));
                }
                let charge_id = charge["id"].as_str().ok_or("omise: missing charge id")?;
                let qr_url = charge["scannable_code"]["image"]["download_uri"]
                    .as_str()
                    .or_else(|| source["scannable_code"]["image"]["download_uri"].as_str())
                    .ok_or("omise: missing QR image URL")?
                    .to_string();

                Ok(PaymentIntent { provider_ref: charge_id.to_string(), qr_url })
            }
        }
    }

    /// ตรวจสอบ webhook: re-fetch event จาก Omise API ด้วย secret แล้วดึงสถานะ charge จริง
    /// คืน (charge_id, omise_status)
    pub async fn verify_webhook(&self, event_id: &str) -> Result<(String, String), String> {
        match self {
            Self::Mock => Err("mock provider ไม่รับ omise webhook".into()),
            Self::Omise { secret, client } => {
                let ev: Value = client
                    .get(format!("{OMISE_API}/events/{event_id}"))
                    .basic_auth(secret, Some(""))
                    .send()
                    .await
                    .map_err(|e| format!("omise event: {e}"))?
                    .json()
                    .await
                    .map_err(|e| format!("omise event json: {e}"))?;
                if ev["object"].as_str() != Some("event") {
                    return Err(format!("omise event error: {ev}"));
                }
                let charge = &ev["data"];
                let charge_id = charge["id"].as_str().ok_or("omise event: missing charge id")?;
                let status = charge["status"].as_str().ok_or("omise event: missing status")?;
                Ok((charge_id.to_string(), status.to_string()))
            }
        }
    }
}

/// อัปเดตสถานะโดเนตแบบ idempotent + ป็อบอัพครั้งเดียว (ใช้ร่วมกับ mock webhook)
pub async fn settle(
    db: &SqlitePool,
    tx: &tokio::sync::broadcast::Sender<std::sync::Arc<crate::DonationEvent>>,
    payment_ref: &str,
    new_status: &str,
) -> Result<(), String> {
    let rows = sqlx::query(
        "UPDATE donations SET status = ?, paid_at = ? WHERE payment_ref = ? AND status = 'pending'",
    )
    .bind(new_status)
    .bind(crate::now_millis())
    .bind(payment_ref)
    .execute(db)
    .await
    .map_err(|e| format!("db error: {e}"))?;

    if rows.rows_affected() > 0 && new_status == "paid" {
        let row = sqlx::query(
            "SELECT user_id, id, donor_name, amount, message, sound, status, payment_ref FROM donations WHERE payment_ref = ?",
        )
        .bind(payment_ref)
        .fetch_one(db)
        .await
        .map_err(|e| format!("db error: {e}"))?;
        let d = crate::row_into_donation(&row);
        let _ = tx.send(std::sync::Arc::new(crate::DonationEvent::from(&d)));
    }
    Ok(())
}
