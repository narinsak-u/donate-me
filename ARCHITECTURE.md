# 🏗️ Donate Me — High-Level System Design

ระบบรับโดเนตสำหรับสตรีมเมอร์: ผู้ชมโดเนตผ่านหน้าเว็บ → ระบบยืนยันการชำระเงิน → ป็อบอัพ + เสียงเด้งบน OBS ของสตรีมเมอร์แบบเรียลไทม์ พร้อมแดชบอร์ดจัดการ

---

## 1. ภาพรวมสถาปัตยกรรม (High-Level Diagram)

```
 ┌──────────┐      ┌─────────────────────────────────────────────┐
 │  ผู้ชม    │      │              BACKEND (API Server)           │
 │ (Viewer) │      │                                             │
 └────┬─────┘      │  ┌───────────┐  ┌──────────┐  ┌──────────┐  │     ┌──────────┐
      │  1.กรอกฟอร์ม │  │ Auth API │  │ Donation │  │ Payment  │  │     │ DATABASE │
      │  โดเนต      │  │ (JWT)    │  │   API    │  │ Service  │◄─┼────►│PostgreSQL│
      ▼            │  └───────────┘  └────┬─────┘  └────┬─────┘  │     └──────────┘
 ┌──────────┐      │                      │             │        │
 │  Payment │  2.สร้าง │                      ▼             ▼        │     ┌──────────┐
 │ Provider │◄────┼────── │ ┌──────────────┐ ┌────────────────┐    │────►│  Redis   │
 │ (Omise / │      │      │ │ Donation DB  │ │ PromptPay QR / │    │     │ (Queue + │
 │  2C2P /  │      │      │ └──────────────┘ │ Credit Card    │    │     │  Cache)  │
 │  PayPal) │      │      └────────────────┘ └────────────────┘    │     └──────────┘
 └────┬─────┘      │                      ▲                      │
      │ 3.Webhook  │                      │ 5.Push realtime      │
      │ ยืนยันเงิน   │                      ▼ (SSE/WebSocket)      │
      └───────────►└──────────────────────────────────────────────┘
                             │                          │
              ┌──────────────┴───────────┐  ┌───────────┴────────────┐
              ▼                          ▼  ▼                        ▼
      ┌──────────────┐         ┌──────────────────┐        ┌────────────────┐
      │ Alert Overlay│         │Streamer Dashboard│        │  Public Page   │
      │ (OBS Browser│         │ (จัดการ/ดูสถิติ/  │        │ (หน้าโปรไฟล์    │
      │  Source)     │         │  ตั้งค่าเสียง/ธีม) │        │  สตรีมเมอร์)    │
      └──────────────┘         └──────────────────┘        └────────────────┘
```

---

## 2. Components หลัก

| Component | หน้าที่ | เทคโนโลยีแนะนำ |
|---|---|---|
| **Public Page** | หน้าโปรไฟล์สตรีมเมอร์ + ฟอร์มโดเนต `donate.me/{username}` | Next.js / Nuxt (SSR) |
| **Donation API** | สร้างโดเนต, ตรวจ validation, anti-spam | Node.js (Express/Nest) |
| **Payment Service** | สร้าง PromptPay QR / บัตรเครดิต / TrueMoney, รับ webhook ยืนยันเงิน | Omise, 2C2P, Stripe |
| **Realtime Gateway** | Push event โดเนตไป overlay + dashboard | SSE (ง่าย) หรือ WebSocket |
| **Alert Overlay** | หน้าโปร่งใสสำหรับ OBS แสดงป็อบอัพ/เสียง/TTS | Static HTML + JS |
| **Streamer Dashboard** | สถิติ, ประวัติโดเนต, ตั้งค่าธีม/เสียง/เป้าหมาย, ทดสอบแจ้งเตือน | SPA (React/Vue) |
| **Auth Service** | สมัคร/ล็อกอินสตรีมเมอร์ | JWT / OAuth (Google, Twitch) |
| **Database** | เก็บโดเนต, ผู้ใช้, การตั้งค่า | PostgreSQL |
| **Cache/Queue** | rate-limit, job queue (ส่งอีเมลขอบคุณ, TTS), pub/sub realtime | Redis |

---

## 3. Donation Flow (ลำดับเหตุการณ์)

```
Viewer                API            Payment          Redis/DB         Overlay(OBS)
  │  1. กรอกฟอร์ม     │                │                 │                 │
  ├──────────────────►│                │                 │                 │
  │                   │ 2. สร้าง charge │                 │                 │
  │                   ├───────────────►│                 │                 │
  │  3. แสดง PromptPay QR              │                 │                 │
  │◄──────────────────┤                │                 │                 │
  │  4. สแกนจ่ายเงิน  │                │                 │                 │
  ├───────────────────────────────────►│                 │                 │
  │                   │ 5. Webhook "payment_success"      │                 │
  │                   │◄───────────────┤                 │                 │
  │                   │ 6. บันทึก DB (status=paid)        │                 │
  │                   ├────────────────────────────────► │                 │
  │                   │ 7. Publish event ไปทาง SSE       │                 │
  │                   ├──────────────────────────────────────────────────► │
  │                   │                │                 │   8. ป็อบอัพ +  │
  │                   │                │                 │      เสียง + TTS│
```

**หลักการสำคัญ:** overlay ไม่เชื่อใจฝั่ง client — ป็อบอัพเด้ง **เฉพาะเมื่อ webhook จากผู้ให้บริการชำระเงินยืนยันว่าได้เงินจริง** เท่านั้น (กันโดเนตปลอมกดทดสอบเอง)

---

## 4. Data Model (หลัก ๆ)

```
users        (id, username, email, password_hash, promptpay_id, created_at)
donations    (id, user_id, donor_name, amount, message, sound, status[pending|paid|failed],
              payment_ref, created_at, paid_at)
settings     (user_id, theme, goal_amount, alert_duration, tts_enabled, custom_css)
             └── เก็บเป็น JSONB ได้เลย
```

---

## 5. Realtime: SSE vs WebSocket

| | SSE (แนะนำ) | WebSocket |
|---|---|---|
| ความซับซ้อน | ต่ำ (HTTP ธรรมดา) | สูงกว่า |
| ทิศทาง | server → client ทางเดียว (พอดีกับงานนี้) | สองทาง |
| OBS เข้ากันได้ | ✅ Browser Source รองรับ | ✅ |
| ต่อยอดเป็น multi-server | ใช้ Redis Pub/Sub กระจาย | เช่นเดียวกัน |

โดเนตเป็น event ทางเดียว server→client → **SSE เพียงพอและเรียบง่ายกว่า**

---

## 6. ความปลอดภัย (ต้องมีใน production)

1. **Webhook signature verification** — ตรวจ HMAC signature จาก Omise/Stripe กัน webhook ปลอม
2. **Idempotency** — ป้องกัน webhook ซ้ำทำให้ป็อบอัพเด้ง 2 ครั้ง (เช็ค `payment_ref` ซ้ำ)
3. **Rate limiting + validation** — กัน spam โดเนตชื่อ/ข้อความยาว ๆ (บล็อกคำหยาบด้วย word filter)
4. **Sanitize ข้อความ** — ห้าม inject HTML/JS ลง overlay ของสตรีมเมอร์ (XSS)
5. **Auth สำหรับ dashboard/ตั้งค่า** — JWT + ownership check ทุก endpoint

---

## 7. Roadmap การพัฒนา

| Phase | งาน | สถานะใน repo นี้ |
|---|---|---|
| 0 | Prototype static (BroadcastChannel) | ✅ เดิม |
| 1 | Node backend + mock PromptPay + SSE + dashboard | ✅ `server.js` |
| 2 | ต่อ Omise/2C2P จริง + webhook + DB (Postgres) | ⬜ ถัดไป |
| 3 | ระบบสมาชิก + หน้าโปรไฟล์ `/username` + ตั้งค่าธีม/เป้าหมาย | ⬜ |
| 4 | ฟีเจอร์พิเศษ: TTS ทุกภาษา, media alert (รูป/วิดีโอ/บทเพลง), top donator board, subscription | ⬜ |
| 5 | Scale: Docker, Redis queue, CDN, ระบบส่วนแบ่ง/ถอนเงิน (ยากสุด — ต้องมี KYC) | ⬜ |
