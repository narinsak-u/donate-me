# 📋 แผน Implement โปรเจค Donate Me (ฉบับสมบูรณ์)

**Stack:** Backend = **Rust (Axum)** · Frontend = **Vue 3 + TypeScript**

แผนงานแบ่งเป็น 6 Phase ไล่จาก MVP ไปจนถึง production — แต่ละ phase จบแล้ว **ใช้งานได้จริงและทดสอบผ่าน** ก่อนไป phase ถัดไป

---

## 📁 โครงสร้างโปรเจคเป้าหมาย

```
donate-me/
├── ARCHITECTURE.md            # เอกสารสถาปัตยกรรม
├── PLAN.md                    # แผนนี้
├── docker-compose.yml         # Postgres + Redis + api + web
├── .env.example               # DATABASE_URL, JWT_SECRET, OMISE_SECRET_KEY, ...
│
├── server/                    # ===== Backend: Rust =====
│   ├── Cargo.toml             # axum, tokio, sqlx, serde, jsonwebtoken,
│   │                          # tower-http (cors, rate limit), redis, tracing
│   ├── migrations/            # sqlx migrations (users, donations, settings)
│   └── src/
│       ├── main.rs            # startup: router, state, graceful shutdown
│       ├── config.rs          # อ่าน env (dotenv)
│       ├── state.rs           # AppState { db pool, redis, config }
│       ├── error.rs           # enum AppError → HTTP response (thiserror)
│       ├── db/mod.rs          # sqlx Pool + repository functions
│       ├── routes/
│       │   ├── mod.rs         # Router mounting
│       │   ├── auth.rs        # สมัคร/ล็อกอิน (JWT, bcrypt → argon2)
│       │   ├── donations.rs   # POST /api/donate, GET ประวัติ/สถิติ
│       │   ├── webhooks.rs    # POST /webhooks/omise (ตรวจ HMAC signature)
│       │   ├── settings.rs    # GET/PUT การตั้งค่า overlay
│       │   ├── public.rs      # GET /api/u/:username (ข้อมูลโปรไฟล์)
│       │   └── events.rs      # GET /events/:userId → SSE stream
│       ├── services/
│       │   ├── payment.rs     # เรียก Omise API (reqwest): PromptPay QR, charge
│       │   ├── broadcaster.rs # publish donation event → Redis Pub/Sub
│       │   └── sanitize.rs    # escape HTML + word filter (กัน XSS บน overlay)
│       ├── middleware/
│       │   ├── auth.rs        # extract + verify JWT, ownership check
│       │   └── rate_limit.rs  # tower governor ต่อ IP
│       └── models/            # struct User, Donation, Settings (derive sqlx::FromRow)
│
└── web/                       # ===== Frontend: Vue 3 + TypeScript + Vite =====
    ├── package.json           # vue, vue-router, pinia, axios
    ├── tsconfig.json          # strict: true
    ├── vite.config.ts
    └── src/
        ├── main.ts
        ├── router/index.ts    # typed routes
        ├── stores/            # Pinia: auth.ts, donation.ts, settings.ts
        ├── api/               # ไคลเอนต์ API แบบมี type
        │   ├── client.ts      # axios instance + interceptor token
        │   └── types.ts       # interface ตรงกับ Rust models 1:1 (single source: docs/api.md)
        ├── composables/
        │   └── useAlertStream.ts   # EventSource wrapper (SSE) แบบ typed + auto-reconnect
        ├── pages/
        │   ├── DonatePage.vue      # /u/:username
        │   ├── PayPage.vue         # หน้า QR รอชำระเงิน
        │   └── Dashboard/
        │       ├── Overview.vue    # สถิติ + กราฟ (typed Chart.js)
        │       ├── Donations.vue
        │       └── Settings.vue    # ตั้งค่า + live preview
        └── overlay/                # ⚠️ build แยกเป็น standalone (entry ต่างหาก)
            └── main.ts             # ไม่เข้า SPA router, โหลดเบาสำหรับ OBS
```

**จุดต่อ Rust↔TS:** กำหนด API contract ใน `docs/api.md` ก่อนเขียนโค้ด แล้วสร้าง type ทั้งสองฝั่งให้ตรงกัน (ระยะยาวดู `ts-rs` เพื่อ export struct Rust → TS interface อัตโนมัติ)

---

## Phase 1 — MVP แบบมีเงินจริง 🎯 (สัปดาห์ 1–2)

> เป้าหมาย: สตรีมเมอร์คนเดียว (ตัวเอง) ใช้รับโดเนตจริงได้

- [ ] **1.1** Scaffold: `cargo new` + Axum server ตอบ `/healthz` + sqlx migrate 3 ตาราง
  (`users`, `donations`, `settings` ตาม data model ใน ARCHITECTURE.md)
- [ ] **1.2** ต่อ Omise (Sandbox ก่อน) — `services/payment.rs`
  - `POST /api/donate` → สร้าง PromptPay source + QR → คืน URL QR
  - `POST /webhooks/omise` → ตรวจ HMAC signature → อัปเดต status=paid
- [ ] **1.3** Idempotency: UNIQUE `payment_ref` — webhook ซ้ำไม่เด้งป็อบอัพซ้ำ (จัดการใน transaction เดียว)
- [ ] **1.4** หน้า `PayPage.vue`: แสดง QR + สถานะ "รอชำระเงิน..." (polling `GET /api/donate/:id`)
- [ ] **1.5** SSE ใน Axum: `GET /events/:userId` (Stream จาก tokio broadcast channel) — overlay เปลี่ยนจาก BroadcastChannel มาเป็น `useAlertStream.ts`
- [ ] **1.6** ทดสอบ end-to-end บน Omise Sandbox + เปิด OBS ทดสอบจริง

**ส่งมอบ:** รับโดเนตจริงได้ (สตรีมเมอร์รายเดียว, แอดมินเพิ่มมือใน DB)

---

## Phase 2 — ระบบสมาชิก + โปรไฟล์สาธารณะ 👤 (สัปดาห์ 3)

- [ ] **2.1** Auth ใน Rust: สมัคร/ล็อกอิน — argon2 hashing + JWT (`jsonwebtoken` crate)
      + Login with Twitch/Google (OAuth)
- [ ] **2.2** หน้า `/u/:username` (Vue + TS): ดึงชื่อ, รูป, goal จาก API — ฟอร์มโดเนตผูก user นั้น
- [ ] **2.3** เช็ค username ซ้ำ, reserve ชื่อพิเศษ, slug ปลอดภัย (validate ฝั่ง Rust เสมอ)
- [ ] **2.4** Middleware ownership: แก้ settings ได้เฉพาะเจ้าของ (auth extractor ใน Axum)

**ส่งมอบ:** หลายสตรีมเมอร์ใช้ร่วมกันได้ แต่ละคนมีลิงก์ + overlay ของตัวเอง

---

## Phase 3 — Streamer Dashboard 📊 (สัปดาห์ 4–5)

- [ ] **3.1** Overview: ยอดวันนี้/เดือน/ทั้งหมด, กราฟ 30 วัน (Chart.js + TS types), goal progress
- [ ] **3.2** ตารางประวัติ: ค้นหา, กรองวันที่, export CSV, ซ่อน/ปักหมุดข้อความ
      (export CSV ทำฝั่ง Rust เพื่อลดภาระ client)
- [ ] **3.3** ตั้งค่า Alert + **live preview**: ธีมสี/ฟอนต์, ระยะเวลา, ตำแหน่งบนจอ
      - อัปโหลดเสียงเอง: รับผ่าน Axum `multipart`, จำกัด 2MB + ตรวจ MIME ฝั่ง Rust
      - ปุ่ม "ทดสอบแจ้งเตือน" ส่ง event ผ่าน SSE จริง
- [ ] **3.4** ตั้งค่า TTS: เปิด/ปิด, เลือกเสียง, กรองคำ, จำกัดความยาวอ่าน
- [ ] **3.5** Custom CSS box — ตรวจ/sanitize ฝั่ง Rust ก่อนเก็บ

**ส่งมอบ:** สตรีมเมอร์บริหารช่องตัวเองได้ครบ ไม่ต้องแตะโค้ด

---

## Phase 4 — ความปลอดภัย + Production 🔒 (สัปดาห์ 6)

- [ ] **4.1** Rate limit: `tower-governor` (Redis-backed) — ฟอร์มโดเนต 5 ครั้ง/นาที/IP
- [ ] **4.2** Word filter (TH/EN) ฝั่ง Rust + แบนชื่อผู้โดเนต + ปุ่มล็อกข้อความ
- [ ] **4.3** XSS: serialize ข้อความเป็น plain text เสมอ (`serde` ไม่ยอม HTML ผ่าน), overlay render เป็น text node
- [ ] **4.4** Monitoring: `tracing` + Sentry Rust, log webhook ทุกตัว, `/healthz`
- [ ] **4.5** Docker multi-stage (builder → `distroless`/`debian-slim` ~20MB) + compose: api, web, Postgres, Redis, Caddy HTTPS
- [ ] **4.6** Deploy VPS/Railway + โดเมนจริง + nightly backup DB

**ส่งมอบ:** ระบบพร้อมเปิดให้คนทั่วไปใช้

---

## Phase 5 — ฟีเจอร์เสริม ✨ (สัปดาห์ 7–8)

- [ ] Top Donator leaderboard (หน้า public + widget overlay)
- [ ] Media alert: แนบรูป/GIF ตามยอดขั้นต่ำที่กำหนด
- [ ] เอฟเฟกต์ตามระดับยอด (20/100/500 บาท = เอฟเฟกต์ต่างกัน)
- [ ] Badge สะสมให้ผู้โดเนต
- [ ] อีเมลขอบคุณอัตโนมัติ — background worker (tokio task + Redis queue)

---

## Phase 6 — ระบบการเงิน 💰 (หลังมี user จริง ค่อยทำ)

- [ ] ถอนเงินให้สตริมเมอร์: **(ก)** Omise จ่ายเข้าบัญชีแยก per-user ตั้งแต่ต้น (ง่ายสุด) หรือ **(ข)** รวมศูนย์แล้วโอนเอง (ต้องบัญชีบริษัท + ledger + ภาษี)
- [ ] ค่าธรรมเนียมโปร่งใส: แสดงส่วนแบ่งบนหน้าโดเนตก่อนจ่าย
- [ ] Ledger audit log append-only ทุกการเคลื่อนไหวเงิน (เหมาะกับ transaction ของ Postgres + type-safety ของ Rust มาก)

---

## ✅ Definition of Done (ทุก Phase)

1. ทดสอบ end-to-end ผ่าน (ฟอร์ม → จ่าย sandbox → เด้ง overlay)
2. `cargo clippy` + `cargo test` ผ่าน (Rust), `vue-tsc --noEmit` + vitest ผ่าน (Frontend)
3. ไม่มี secret ในโค้ด — ใช้ `.env` ทั้งหมด
4. API มีเอกสาร `docs/api.md` อัปเดตทุกครั้งที่เพิ่ม endpoint
5. Overlay ทดสอบบน OBS จริงทุกครั้งที่แตะ frontend

---

## 🧱 Stack สรุป

| ชั้น | เทคโนโลยี | เหตุผล |
|---|---|---|
| Backend | **Rust + Axum + Tokio** | type-safe, async SSE ทำงานได้ดี, binary เดียว deploy ง่าย, ปลอดภัยสูงกับงานการเงิน |
| ORM/DB | **sqlx + PostgreSQL** | SQL ตรวจตอน compile, migration ในตัว |
| Auth | argon2 + jsonwebtoken | มาตรฐานสูงกว่า bcrypt |
| Frontend | **Vue 3 + TypeScript (strict) + Vite** | SPA dashboard + overlay แยก build, Pinia + typed API client |
| Realtime | SSE (Axum stream) + Redis Pub/Sub | push ทางเดียว พอและเรียบง่าย |
| ชำระเงิน | Omise (PromptPay + บัตร) | รองรับไทยครบ, webhook เสถียร, เรียกผ่าน reqwest |
| Deploy | Docker multi-stage + Caddy บน VPS / Railway | binary Rust เล็กมาก, ควบคุมค่าใช้จ่ายได้ |
