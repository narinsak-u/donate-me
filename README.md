# Donate Me ❤️

ระบบรับโดเนตสำหรับสตรีมเมอร์ — ผู้ชมโดเนตผ่านหน้าเว็บ → ยืนยันการชำระเงิน (PromptPay) → ป็อบอัพ + เสียงเด้งบนหน้าจอ stream (OBS) แบบเรียลไทม์ พร้อมแดชบอร์ดจัดการสำหรับสตรีมเมอร์

**Stack:** Backend **Rust (Axum + sqlx + SQLite)** · Frontend **Vue 3 + TypeScript (Vite, bun)** · Realtime **SSE** · Deploy **Docker + Caddy**

> 💳 สถานะการชำระเงิน: ระบบรองรับ 2 โหมด — **Mock** (ค่าเริ่มต้น ไม่มีการตัดเงิน ใช้พัฒนา/ทดสอบ) และ **Omise จริง** (เปิดโดยใส่ `OMISE_SECRET_KEY` ใน `.env` — แนะนำเริ่มจาก test mode ที่ไม่ตัดเงินจริงก่อน)

---

## ✨ ฟีเจอร์

### ฝั่งผู้ชม (Viewer)
- หน้าโดเนตสาธารณะ `/?u=<username>` — แสดงชื่อสตรีมเมอร์, ปุ่มยอดด่วน (20/50/100/500), ข้อความอวยพร
- เลือกเสียงแจ้งเตือน: 🔔 กระดิ่ง · 🪙 เหรียญ · 🎺 โห่ร้อง · 🗣️ TTS อ่านข้อความภาษาไทย
- ชำระผ่าน **PromptPay QR** (หน้า "รอชำระเงิน" อัปเดตสถานะอัตโนมัติ + countdown QR 15 นาที)
- หน้าโดเนตแสดง **🏆 Top Donators** (รวมยอดสะสม 5 อันดับแรก) — สตรีมเมอร์เปิด/ปิดได้

### ฝั่ง Overlay บน OBS
- หน้าโปร่งใส `/overlay.html?token=<JWT>` — ใส่เป็น Browser Source (1920×1080)
- ป็อบอัพเด้งทันทีเมื่อจ่ายเงินสำเร็จ: ชื่อ + ยอด + ข้อความ + คอนเฟตติ
- **เอฟเฟกต์ตามระดับยอด:** ฿100+ ใหญ่ขึ้น/คอนเฟตติเยอะขึ้น, ฿500+ = "SUPER CHAT!" ขนาดใหญ่สุด
- 4 ธีมสี (ชมพู/น้ำเงิน/เขียว/ดำ), รูป/GIF ประกอบ (media alert), เสียงอัปโหลดของตัวเอง หรือเสียงสังเคราะห์/TTS
- ทดสอบโดยไม่ต้องโดเนต: ปุ่ม "ทดสอบแจ้งเตือน" ใน Dashboard หรือเรียก `window.testAlert()` ใน console

### ฝั่งสตรีมเมอร์
- สมัคร/ล็อกอิน (argon2 + JWT 7 วัน) — แต่ละบัญชีมีลิงก์โดเนตและ overlay ของตัวเอง
- **Dashboard 3 แท็บ:**
  - *ภาพรวม* — ยอดวันนี้/เดือนนี้/รวม/จำนวนครั้ง + กราฟ 30 วัน + แถบความคืบหน้าเป้าหมาย
  - *ประวัติโดเนต* — ค้นหา, pagination, ซ่อนข้อความก่อกวน, Export CSV
  - *ตั้งค่า* — ธีม, เป้าหมาย, ระยะเวลาป็อบอัพ, TTS, ข้อความแจ้งเตือน (template `{name}` `{amount}`), อัปโหลดเสียง (mp3/wav/ogg ≤ 2MB), รูป/GIF, เปิด/ปิด leaderboard + ปุ่มทดสอบ

### ความปลอดภัย
- Rate limit ต่อ IP: โดเนต 5 ครั้ง/นาที, login/สมัคร 10 ครั้ง/นาที (เกิน = HTTP 429)
- กรองคำหยาบ TH/EN อัตโนมัติ (`fu*k`, `ก**`) + ทำความสะอาด control characters
- กัน XSS: ข้อความผู้ใช้ render ด้วย `textContent` เท่านั้น, URL รูปต้องเป็น `https://` เท่านั้น
- Webhook: mock มี HMAC-SHA256 signature / Omise ตรวจโดย **re-fetch event จาก API** (วิธีมาตรฐานของ Omise) + idempotent (โดเนตเด้งป็อบอัพครั้งเดียว) + retry 5 รอบเมื่อ DB ล้มชั่วคราว
- อัปโหลดเสียง: จำกัด 2MB + MIME allowlist

---

## 🏗️ ภาพรวมสถาปัตยกรรม

```
ผู้ชม ──► หน้าโดเนต (Vue) ──► POST /api/donate ──► PaymentProvider ──► Omise จริง หรือ Mock QR
                                        │                                    │
                                        ▼                                    ▼
                                  SQLite (pending)               ผู้ชมสแกน QR จ่ายเงิน
                                                                             │
Overlay (OBS) ◄── SSE /events (ป็อบอัพ) ◄── settle() ◄── webhook ◄───────────┘
                                   (idempotent + retry)      /webhooks/omise หรือ /mock/webhook

Streamer Dashboard ◄── REST API (stats/history/settings) ──► SQLite
```

- **PaymentProvider** เลือกอัตโนมัติจาก `.env`: มี `OMISE_SECRET_KEY` → Omise / ไม่มี → Mock
- เมื่อใช้ Omise จริง route mock ทั้งหมดจะถูกปิดอัตโนมัติ
- รายละเอียดเพิ่มเติม: [ARCHITECTURE.md](ARCHITECTURE.md) · แผนงาน: [PLAN.md](PLAN.md) / [PLAN-PHASES.md](PLAN-PHASES.md)

### โครงสร้างโปรเจค

```
donate-me/
├── server/                  # Backend (Rust + Axum)
│   ├── src/
│   │   ├── main.rs          # routes, SSE, webhook handlers
│   │   ├── auth.rs          # สมัคร/ล็อกอิน (argon2 + JWT), AuthUser extractor
│   │   ├── users.rs         # โปรไฟล์สาธารณะ, settings, top donators
│   │   ├── dashboard.rs     # สถิติ, ประวัติ, CSV, อัปโหลดเสียง
│   │   ├── payment.rs       # PaymentProvider (Mock/Omise) + settle()
│   │   ├── mock_pay.rs      # หน้า Mock Bank + QR ปลอม
│   │   ├── rate_limit.rs    # sliding window ต่อ IP
│   │   └── sanitize.rs      # กรองคำหยาบ
│   └── migrations/          # sqlx migrations
├── web/                     # Frontend (Vue 3 + TS + Vite)
│   └── src/
│       ├── pages/           # DonatePage, PayPage, AuthPage, Dashboard
│       ├── overlay/main.ts  # standalone overlay สำหรับ OBS (build แยก)
│       ├── api/auth.ts      # API client + session (localStorage)
│       └── components/      # DonationChart
├── Dockerfile               # multi-stage (bun → rust → slim runtime)
├── docker-compose.yml       # app + Caddy (HTTPS อัตโนมัติ)
├── Caddyfile                # reverse proxy config
└── .env.example             # template ตัวแปร environment
```

---

## 🚀 Setup — แบบ Dev (แนะนำสำหรับเริ่มต้น)

### สิ่งที่ต้องมี
- **Rust** (จาก [rustup.rs](https://rustup.rs)) — ตรวจสอบ: `cargo --version`
- **bun** (จาก [bun.sh](https://bun.sh)) — ตรวจสอบ: `bun --version`

### ขั้นตอน

```bash
# 1) ติดตั้ง dependencies ฝั่ง frontend + build
cd web
bun install
bun run build          # ผลลัพธ์อยู่ที่ web/dist

# 2) รัน backend (serve ทั้ง API และหน้าเว็บที่ port 3000)
cd ../server
cargo run
```

เปิด **http://localhost:3000** — เสร็จแล้ว!

> 💡 `server/donations.db` จะถูกสร้างอัตโนมัติพร้อม migration ครั้งแรกที่รัน

### โหมด hot reload (แก้โค้ดแล้วเห็นผลทันที)

```bash
cd server && cargo run                      # terminal 1: API ที่ :3000
cd web && bun run dev                       # terminal 2: Vite ที่ :5173 (proxy /api ไป :3000)
```

เปิด http://localhost:5173 (หน้า :3000 จะไม่อัปเดตถ้ายังไม่ `bun run build`)

---

## 🧪 ลองใช้ครบรอบใน 2 นาที (โหมด Mock)

1. เปิด http://localhost:3000 → ไปที่ `/#/auth` → **สมัครสมาชิก** (username เช่น `mychannel`, รหัสผ่าน 8+ ตัว)
2. หลังสมัครจะพาไป **Dashboard** — คัดลอก "ลิงก์รับโดเนต" (`.../?u=mychannel`)
3. เปิดลิงก์นั้นในแท็บใหม่ → กรอกชื่อ/ยอดเงิน → **โดเนตเลย!** → ได้ QR → กดปุ่ม **"🧪 จำลองการชำระเงิน"** (เปิดหน้า Mock Bank) → กด **"✅ จำลองว่าจ่ายสำเร็จ"**
4. กลับมา Dashboard → แท็บ *ตั้งค่า* → คัดลอก **ลิงก์ Overlay** (`/overlay.html?token=...`) เปิดในแท็บใหม่ → จะเห็นป็อบอัพเด้งพร้อมเสียงทันที
5. ใช้จริงบน OBS: เพิ่ม **Browser Source** → วางลิงก์ Overlay → กว้าง 1920 × สูง 1080 → ✅ *Shutdown source when not visible* ไม่ต้องติ๊ก (ให้ SSE เชื่อมต่อตลอด)

> ใน Dashboard กดปุ่ม **"▶ ทดสอบแจ้งเตือน"** เพื่อเช็คว่า overlay เด้งได้โดยไม่ต้องโดเนต

---

## 💳 เปิดใช้เงินจริง (Omise)

1. สมัครบัญชี [omise.co](https://www.omise.co) (มี sandbox/test mode ฟรี)
2. เอา **Secret Key** จาก Dashboard (test = `skey_test_...` — ยังไม่ตัดเงินจริง)
3. แก้ `.env`:
   ```env
   OMISE_SECRET_KEY=skey_test_xxxxxxxxxxxx
   ```
4. รีสตาร์ท server — log ต้องขึ้น `💳 PaymentProvider = Omise`
5. ใน Omise Dashboard → Webhooks → เพิ่ม URL: `https://<โดเมนคุณ>/webhooks/omise`
6. ทดสอบด้วยบัตร test ของ Omise / สแกน PromptPay จริง (test mode) → ตรวจว่า overlay เด้ง
7. เมื่อพร้อมใช้งานจริง เปลี่ยนเป็น `skey_live_...`

**เกิดอะไรขึ้นเมื่อเปิด Omise:** ระบบสร้าง PromptPay QR จริง → ผู้ชมสแกนจ่าย → Omise ยิง webhook มาที่ `/webhooks/omise` → server re-fetch event จาก Omise API เพื่อยืนยัน → สถานะเป็น paid → ป็อบอัพเด้ง (route จำลองทั้งหมดถูกปิดอัตโนมัติ)

---

## 🐳 Setup — แบบ Production (Docker)

```bash
# 1) สร้าง .env จาก template แล้วแก้ค่า
cp .env.example .env
# สร้าง JWT_SECRET ที่ปลอดภัย:
openssl rand -hex 32      # คัดลอกผลลัพธ์ใส่ JWT_SECRET ใน .env

# 2) แก้ Caddyfile — เปลี่ยน donation.example.com เป็นโดเมนจริง
#    (ชี้ DNS A record ของโดเมนมาที่ server นี้ก่อน — Caddy ออกใบ cert HTTPS ให้เอง)

# 3) build และรัน
docker compose up -d --build

# 4) ดู log / ตรวจสถานะ
docker compose logs -f app
curl https://<โดเมน>/healthz
```

ข้อมูลถาวรอยู่ใน Docker volumes: `app-data` (ฐานข้อมูล SQLite) และ `app-uploads` (ไฟล์เสียง) — backup ด้วย `docker run --rm -v donate-me_app-data:/data -v $(pwd):/backup alpine cp /data/donations.db /backup/`

---

## ⚙️ Environment Variables ทั้งหมด

| ตัวแปร | ค่าเริ่มต้น (dev) | คำอธิบาย |
|---|---|---|
| `DATABASE_URL` | `sqlite://donations.db?mode=rwc` | ตำแหน่งฐานข้อมูล SQLite |
| `JWT_SECRET` | `jwt-dev-secret-change-me` | **ต้องเปลี่ยนใน production** — ใช้ `openssl rand -hex 32` |
| `MOCK_WEBHOOK_SECRET` | `mock-secret-change-me` | ลับ HMAC ของ mock webhook |
| `OMISE_SECRET_KEY` | *(ว่าง = Mock)* | ใส่ `skey_test_...`/`skey_live_...` เพื่อเปิดเงินจริง |
| `DEFAULT_USER_ID` | `default` | สตรีมเมอร์ปลายทางเมื่อโดเนตไม่ระบุ `?u=` |
| `RUST_LOG` | `info` | log level |

---

## 🧑‍💻 คำสั่งที่ใช้บ่อย

```bash
# รัน tests ฝั่ง backend
cd server && cargo test

# ตรวจ lint ฝั่ง frontend (type check อัตโนมัติใน bun run build)
cd web && bunx vue-tsc --noEmit

# รีเซ็ตฐานข้อมูล (ลบแล้ว migration สร้างใหม่)
rm server/donations.db

# ทดสอบ API ด้วย curl
curl http://localhost:3000/healthz
curl -X POST http://localhost:3000/api/test-alert -H "Authorization: Bearer <token>"
```

### ปลายทาง API หลัก

| Method | Path | คำอธิบาย |
|---|---|---|
| POST | `/api/donate` | สร้างรายการโดเนต → คืน QR |
| GET | `/api/donate/:id` | สถานะการชำระเงิน (polling) |
| POST | `/api/auth/register` / `login` | สมัคร / ล็อกอิน (JWT) |
| GET | `/api/u/:username` | โปรไฟล์สาธารณะ |
| GET | `/api/u/:username/top` | Top donators |
| GET/PUT | `/api/me/settings` | ตั้งค่า alert (ต้อง login) |
| GET | `/api/me/stats` | สถิติ + กราฟ 30 วัน |
| GET | `/api/me/donations` / `.csv` | ประวัติ / export |
| POST | `/api/me/alert-sound` | อัปโหลดเสียง (multipart) |
| POST | `/api/test-alert` | ส่ง event ทดสอบไป overlay |
| GET | `/events?token=` | SSE stream ของ overlay |
| POST | `/mock/webhook` · `/webhooks/omise` | ยืนยันการชำระเงิน (mock/จริง) |
| GET | `/healthz` | health check |

---

## ❓ แก้ปัญหาที่เจอบ่อย

| อาการ | วิธีแก้ |
|---|---|
| เปิดเว็บไม่ได้ / 404 | ต้อง `bun run build` ใน `web/` ก่อน (Axum serve จาก `web/dist`) |
| ป็อบอัพไม่เด้ง / ไม่มีเสียง | ต้องเปิด **`/overlay.html` แยกต่างหาก** (แท็บแยกหรือ OBS) — ป็อบอัพ/เสียงแสดงบน overlay เท่านั้น ไม่ได้แสดงบนหน้าโดเนต; overlay แบบไม่ใส่ token ฟังเฉพาะโดเนตของ streamer เริ่มต้น (โดเนตผ่านหน้า `/?u=xxx` จะเด้งเฉพาะ overlay ที่มี token ของ xxx จาก Dashboard); เสียง TTS ต้องมี voice ภาษาไทยในเครื่อง (ไม่มี = เล่นเสียงแตรวงแทนอัตโนมัติ); คลิกหน้า overlay 1 ครั้งเพื่อปลดล็อกเสียง (autoplay policy) |
| เสียงไม่ดังครั้งแรก | Browser ต้องมี interaction ก่อนเล่นเสียง — ใน OBS ตั้ง "Refresh browser when scene becomes active" หรือกด test alert ก่อนไลฟ์ |
| `429 Too Many Requests` | โดน rate limit — รอ 1 นาที (dev: รีสตาร์ท server เพื่อเคลียร์) |
| QR หมดอายุ | PromptPay QR อายุ 15 นาที — โดเนตใหม่ได้เลย |
| พอร์ต 3000 ไม่ว่าง | เปลี่ยนพอร์ตใน `server/src/main.rs` หรือปิด process เดิม (`taskkill //IM donate-me-api.exe //F`) |

---

## 📌 ข้อจำกัด / ขั้นถัดไป

- ฐานข้อมูลเป็น SQLite ต่อ instance เดียว — ถ้า scale หลาย server ให้ย้ายเป็น PostgreSQL + Redis Pub/Sub (ดู PLAN.md Phase 4-6)
- อีเมลขอบคุณอัตโนมัติยังไม่ทำ (ต้องมี SMTP credentials)
- ระบบถอนเงิน/แบ่งรายได้ยังไม่รวมอยู่ในระบบ (เงินเข้าบัญชี Omise ของเจ้าของโดยตรง)
