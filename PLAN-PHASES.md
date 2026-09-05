# 📋 Implementation Plan ราย Phase (Mockup-First — ยังไม่ตัดเงินจริง)

หลักการ: **ทุกอย่างสร้างครบตามสถาปัตยกรรมจริง แต่ Payment Provider เป็น Mock**
เพื่อให้ออกแบบ/ทดสอบ UX และ realtime ได้ครบ ก่อนเสียเงินเชื่อมมตัวจริง (ค่อยสลับตอน Phase 6)

> Mock = สร้าง QR ปลอม + มีหน้า "จำลองการชำระเงิน" กดยืนยันเองได้ + webhook จำลอง
> ออกแบบเป็น `PaymentProvider` trait ให้สลับกับ Omise จริงภายหลังได้โดยไม่แก้โค้ดส่วนอื่น

---

## 🔌 แนวคิดหลัก: PaymentProvider Trait (Rust)

```rust
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_promptpay(&self, amount: i64, ref_id: &str) -> Result<PaymentIntent>;
    // PaymentIntent { qr_image_url, expires_at, provider_ref }
}

// Phase 1–5 ใช้ตัวนี้
pub struct MockProvider { base_url: String }   // สร้าง QR ปลอม + หน้า confirm เอง

// Phase 6 สลับเป็นตัวนี้ (แก้แค่ config)
pub struct OmiseProvider { secret_key: String }
```

Frontend ก็เรียก API เดิมทั้งหมด — สลับ provider ฝั่ง Rust อย่างเดียว

---

# Phase 0 — Scaffold โปรเจค 🏗️ (1 วัน)

| # | งาน | ผลลัพธ์ |
|---|---|---|
| 0.1 | Monorepo setup: `server/` (cargo new, axum, tokio, sqlx, tracing) + `web/` (Vite, Vue 3, TS strict, Pinia, Tailwind) | build ผ่านทั้งคู่ |
| 0.2 | `docker-compose.yml`: Postgres + Redis (ยังไม่มี app container) | `docker compose up` ใช้ได้ |
| 0.3 | Axum hello: `GET /healthz`, cors, request tracing | ยิง Postman เห็น log |
| 0.4 | Vue hello: router + layout ว่าง + `api/client.ts` (axios) | เปิด `localhost:5173` ได้ |
| 0.5 | `docs/api.md` เขียน API contract ของทุก phase ล่วงหน้า | ทีมอ้างอิงเดียวกัน |

✅ **DoD:** `cargo run` + `npm run dev` คุยกันผ่าน `/healthz` ได้

---

# Phase 1 — Donation Flow แบบ Mock 💸 (สัปดาห์ 1)

### Backend (Rust)
| # | งาน |
|---|---|
| 1.1 | Migration: ตาราง `donations` + `settings` (user เดียว hardcode `DEFAULT_USER_ID` ไปก่อน) |
| 1.2 | `POST /api/donate` — validate (ชื่อ ≤50, ยอด 1–100,000, ข้อความ ≤200) → status=`pending` → เรียก `MockProvider.create_promptpay` |
| 1.3 | **Mock QR:** สร้าง QR จริงจาก payload ปลอม `promptpay-mock://{ref}` โดยใช้ crate `qrcode` (สร้าง PNG เอง ไม่พึ่งเน็ต) เก็บที่ `/mock/qr/{ref}.png` |
| 1.4 | **Mock หน้าจ่ายเงิน:** `GET /mock/pay/{ref}` — หน้า HTML คล้ายธนาคาร ("PromptPay Mock", ยอด, ปุ่ม "✅ จ่ายสำเร็จ" / "❌ ยกเลิก") |
| 1.5 | `POST /mock/webhook` — จำลอง webhook: หน้า mock กดยืนยันแล้วเรียก พร้อม HMAC signature ปลอม (ให้ `webhooks.rs` ตรวจ signature จริงจังตั้งแต่ต้น กันลืมตอนสลับของจริง) |
| 1.6 | Idempotency: UNIQUE `payment_ref` + `UPDATE ... WHERE status='pending'` ใน transaction เดียว |
| 1.7 | SSE `GET /events/:userId` — tokio broadcast channel; webhook เสร็จ → publish `donation` event |
| 1.8 | `GET /api/donate/:id` — หน้า PayPage polling สถานะ |

### Frontend (Vue + TS)
| # | งาน |
|---|---|
| 1.9 | `DonatePage.vue` — ดีไซน์จาก prototype เดิม (glassmorphism) + typed form validation |
| 1.10 | `PayPage.vue` — แสดง QR mock + countdown หมดอายุ 15 นาที + polling สถานะ → หาย่ายแล้วแสดง "ขอบคุณ ❤️" + confetti |
| 1.11 | Overlay `overlay/main.ts` — โหลดการตั้งค่าจาก API → `useAlertStream.ts` (SSE + auto-reconnect) → ป็อบอัพ/เสียงสังเคราะห์/คอนเฟตติ (ยก prototype เดิมมาใส่ type) |

### 🧪 ทดสอบ Phase 1
- `cargo test`: validation, idempotency (ยิง webhook ซ้ำ 2 ครั้ง event ต้องออกครั้งเดียว)
- E2E มือ: โดเนต → เปิดแท็บ `/mock/pay/{ref}` กดยืนยัน → overlay เด้ง + เสียง บน OBS จริง

✅ **DoD:** flow ครบตั้งแต่ฟอร์มจนป็อบอัพเด้งบน OBS โดยไม่มีเงินจริง

---

# Phase 2 — Auth + โปรไฟล์สตรีมเมอร์หลายคน 👤 (สัปดาห์ 2)

| # | งาน |
|---|---|
| 2.1 | Migration `users` + `argon2` hash + JWT (`POST /api/auth/register`, `/login`, `GET /api/me`) |
| 2.2 | `middleware/auth.rs` — Axum extractor `AuthUser` ทุก route ที่ต้อง login |
| 2.3 | `GET /api/u/:username` — ข้อมูล public (ชื่อ, รูป, goal, top donators) |
| 2.4 | `PATCH /api/me/settings` — ธีม/เป้าหมาย/เสียง (ownership check) |
| 2.5 | Frontend: `LoginPage`/`RegisterPage` + Pinia `auth.ts` (token + interceptor) |
| 2.6 | `DonatePage` ดึงข้อมูลจาก `/api/u/:username` (ไม่ hardcode แล้ว) — overlay + SSE ผูก `userId` |
| 2.7 | Username validation: `[a-z0-9_]{3,20}` + reserve (`admin`, `api`, `mock`, ...) |
| 2.8 | OAuth Twitch/Google — **เลื่อนได้** ถ้าอยาก ship เร็ว (ใส่ใน backlog) |

🧪 **ทดสอบ:** สมัคร 2 account → โดเนตหา A แล้ว overlay ของ B ต้อง **ไม่** เด้ง (SSE isolation ต่อ userId)

✅ **DoD:** หลายสตรีมเมอร์แยกกันสมบูรณ์

---

# Phase 3 — Streamer Dashboard 📊 (สัปดาห์ 3)

| # | งาน |
|---|---|
| 3.1 | Backend: `GET /api/me/stats` (วัน/เดือน/รวม + series 30 วัน แยกเป็น SQL aggregate), `GET /api/me/donations?from&to&q` + pagination |
| 3.2 | `GET /api/me/donations.csv` — export ฝั่ง Rust |
| 3.3 | Frontend Overview: stat cards + Chart.js (typed) + goal progress bar |
| 3.4 | ตารางประวัติ: filter วันที่, ค้นหา, export, ปุ่ม "ซ่อนข้อความ" (`PATCH /api/me/donations/:id`) |
| 3.5 | Settings + **live preview**: แก้ธีมแล้วเห็น overlay ปลอมเด้งในหน้าจอทันที (component overlay ใช้ตัวเดียวกับ production — สำคัญ อย่าเขียนซ้ำ) |
| 3.6 | ปุ่ม "ทดสอบแจ้งเตือน" → `POST /api/me/test-alert` → publish SSE จริง |
| 3.7 | อัปโหลดเสียง: multipart ใน Axum, จำกัด 2MB, MIME allowlist (`audio/mpeg`, `audio/wav`, `audio/ogg`), เก็บ local `uploads/` (Phase 4 ค่อยย้าย S3/R2) |

🧪 **ทดสอบ:** ทดสอบ alert จาก dashboard เด้งบน OBS + ตั้งธีมแล้ว overlay ใช้ค่าใหม่ทันที (reload)

✅ **DoD:** สตรีมเมอร์จัดการตัวเองได้ครบโดยไม่แตะโค้ด

---

# Phase 4 — Security + Production-Ready 🔒 (สัปดาห์ 4)

| # | งาน |
|---|---|
| 4.1 | Rate limit ด้วย `tower-governor` + Redis: donate 5/min/IP, auth 10/min/IP |
| 4.2 | Word filter TH/EN ใน `sanitize.rs` + blacklist ชื่อ + ตรวจแน่วข้อความโดเนต render เป็น text เท่านั้น (E2E ทดสอบ XSS `<img onerror=...>`) |
| 4.3 | `tracing` structured log + Sentry + `/healthz` + `/metrics` |
| 4.4 | Docker multi-stage (Rust builder → distroless; web → nginx) + compose ครบ 4 service + Caddy HTTPS |
| 4.5 | Deploy ขึ้น staging (Railway หรือ VPS) + backup Postgres รายวัน + `docs/runbook.md` |

🧪 **ทดสอบ:** สคริปต์ flood 100 requests → โดน limit; inject XSS ทุกช่อง → overlay แสดงเป็นข้อความเฉย ๆ

✅ **DoD:** ปล่อยให้คนทั่วไปใช้ได้ปลอดภัย

---

# Phase 5 — ฟีเจอร์เสริม ✨ (สัปดาห์ 5–6)

| # | งาน |
|---|---|
| 5.1 | Leaderboard: `GET /api/u/:username/top` + widget overlay มุมจอ |
| 5.2 | Alert tiers ตามยอด: 20+/100+/500+ = ขนาด/เอฟเฟกต์/ความยาวเสียงต่างกัน (config ใน settings) |
| 5.3 | Media alert: URL รูป/GIF ต่อ tier (allowlist domain กัน SSRF) |
| 5.4 | TTS setting ขยาย: เลือกเสียง/ความเร็ว/จำกัดความยาว + กรองคำก่อนอ่าน |
| 5.5 | E-mail ขอบคุณ: tokio background worker + Redis queue (SMTP ผ่าน `lettre`) |
| 5.6 | หน้า public `/u/:username` จัดรูปแบบสวย + embed แชร์ลงโซเชียล (OG tags) |

✅ **DoD:** ครบฟีเจอร์พื้นฐานเทียบ Streamlabs ได้

---

# Phase 6 — สลับเป็นเงินจริง 💰 (เมื่อพร้อมจริง ๆ)

| # | งาน |
|---|---|
| 6.1 | `OmiseProvider` implement trait เดิม: PromptPay source + charge (test mode ก่อน) |
| 6.2 | `POST /webhooks/omise` — ตรวจ signature จริง + replays จาก Omise |
| 6.3 | ทดสอบบัตร test cards ของ Omise + สแกน PromptPay จริงเงินน้อย ๆ (เช่น 1 บาท) |
| 6.4 | ปิด route mock ทั้งหมด (`cfg(feature = "mock-pay")` — compile ออกใน production build) |
| 6.5 | ลบ/เก็บประวัติ `pending` ค้าง > 24 ชม. (cron ใน tokio task) |
| 6.6 | Ledger + ถอนเงิน (แยก sub-phase ตามแผนหลัก PLAN.md) |

✅ **DoD:** โดเนตจริงเงินเข้าจริง และโค้ด mock ไม่หลุดออกไป production

---

## 🗓️ ไทม์ไลน์รวม

```
W1      W2      W3      W4      W5-6     W7+
Phase0-1 Phase2  Phase3  Phase4  Phase5   Phase6 (ตามความพร้อม)
        └── demo ใช้งานจริงแบบ mock ได้ตั้งแต่นี้ ──┘
```

## 🧪 Definition of Done กลาง (ทุก phase)

1. E2E ผ่าน (ฟอร์ม → mock pay → overlay เด้ง)
2. `cargo clippy && cargo test` ผ่าน, `vue-tsc --noEmit` ผ่าน
3. อัปเดต `docs/api.md` ทุก endpoint ใหม่
4. ทดสอบ overlay บน OBS จริงทุกครั้งที่แตะ frontend
