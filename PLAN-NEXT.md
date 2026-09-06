# PLAN-NEXT — แผนพัฒนาชุดถัดไป (อิง Gap Analysis กับ EasyDonate)

> เป้าหมาย: **สเกลเล็กลงกว่า EasyDonate แต่ใช้งานได้จริง + UI สวยทันสมัย**
> เขียน: 2026-09-06 · ต่อจาก PLAN-PHASES.md (Phase 1–6 เสร็จหมดแล้ว)

## สิ่งที่ตกลงกับเจ้าของโปรเจค

- ✅ ทำ: P0 โอนตรง+ยืนยันสลิป, P1 สถานะวิดเจ็ต/แตก overlay/ธีมหน้าโดเนต, P2 (โควตาแพลน, รายงานภาษี, webhook นักพัฒนา, i18n — ไว้ทำทีหลัง)
- ❌ **ไม่ทำ: คลังเสียงหลายเสียงให้ผู้ชมเลือก** — คงระบบเดิม (ผู้ชมเลือก chime/coin/fanfare/tts + เจ้าของอัปโหลดเสียงเดียวผ่าน `upload_sound`)
- ❌ ไม่ทำ (เกินสเกล): สติกเกอร์, ระบบสังกัด, สมาชิกรายเดือน+โพสต์, ส่งสื่อขึ้นจอ, แชทซัพพอร์ตในเว็บ
- ✅ **Discovery ไม่ต้องทำแล้ว** — commit `e7ce36c` ทำเสร็จแล้ว (`StreamersPage.vue` = หน้าแรก, `GET /api/streamers`, seed ข้อมูลตัวอย่าง)

## หลักการออกแบบที่คงไว้

1. เงิน **ไม่ผ่านบัญชีแพลตฟอร์ม** — ผู้ชมโอนเข้าบัญชีสตรีมเมอร์ตรง (positioning เดียวกับ EasyDonate: "0% ค่าธรรมเนียม")
2. `PaymentProvider` เป็น abstraction เดิม — เพิ่มตัวใหม่ ไม่ทำลายของเดิม (Mock สำหรับ dev, Omise ทางเลือกอนาคต)
3. Alert ขึ้นจอ **หลังเงินยืนยันแล้วเท่านั้น** — reuse `payment::settle()` ที่ idempotent + SSE เดิม
4. ทุกฟิลด์ใหม่มี default ย้อนหลังได้ (migration ADD COLUMN + default) — หน้าเก่าไม่พัง

---

## Phase 7 (P0) — โอนตรง + ยืนยันสลิป (DirectTransfer)

**ทำไมทำก่อน: ตัวเดียวที่ทำให้เว็บ "ใช้หาเงินได้จริง" — ทุกอย่างอื่นเป็นการเพิ่มคุณภาพ**

### 7.1 แนวคิด
สตรีมเมอร์กรอกเบอร์ PromptPay (หรือเลขบัตร 13 หลัก) ใน dashboard → หน้าโดเนตสร้าง **PromptPay QR จริง** (มาตรฐาน EMVCo, ฝังยอดเงิน) → ผู้ชมสแกนแล้วเงินเข้าบัญชีสตรีมเมอร์โดยตรง → ผู้ชมแนบสลิป → ตรวจสอบ → alert ขึ้นจอ

**ลำดับการเลือก provider ต่อการโดเนต** (ที่ `create_donation`):
1. `OMISE_SECRET_KEY` ตั้งไว้ → Omise (เหมือนเดิม)
2. สตรีมเมอร์ปลายทางมี `promptpay_id` → **DirectTransfer** (ใหม่)
3. ไม่มีอะไรเลย → Mock (dev ต่อได้เหมือนเดิม)

### 7.2 Migration `0007_direct_transfer.sql`
```sql
ALTER TABLE settings ADD COLUMN promptpay_id TEXT NOT NULL DEFAULT '';      -- เบอร์โทร 10 หลัก หรือบัตร 13 หลัก
ALTER TABLE settings ADD COLUMN bank_name   TEXT NOT NULL DEFAULT '';      -- แสดง "ช่องทางอื่น" บน PayPage
ALTER TABLE settings ADD COLUMN bank_no     TEXT NOT NULL DEFAULT '';

ALTER TABLE donations ADD COLUMN slip_path    TEXT NOT NULL DEFAULT '';    -- uploads/slips/{random}.png
ALTER TABLE donations ADD COLUMN slip_at     INTEGER;                      -- เวลาแนบสลิป
ALTER TABLE donations ADD COLUMN reviewed_at INTEGER;                      -- เวลาอนุมัติ/ปฏิเสธ
ALTER TABLE donations ADD COLUMN review_note TEXT NOT NULL DEFAULT '';     -- เหตุผลตอนปฏิเสธ
ALTER TABLE donations ADD COLUMN expires_at  INTEGER;                      -- deadline จ่าย (created_at + 15 นาที)
```
สถานะ donation ใหม่ (คอลัมน์ `status` เดิม): `pending` → `awaiting_review` (แนบสลิปแล้ว) → `paid` / `rejected` / `expired`

### 7.3 Backend (`payment.rs` + ไฟล์ใหม่ `slip.rs`)
- [ ] `promptpay.rs` (ใหม่): สร้าง EMVCo payload — TLV tags: `00` version, `01` static/dynamic, `29` (AID `A000000677010111` + promptpay_id), `53` currency 764, `54` amount, `58` TH, `63` **CRC16-CCITT** — แล้วเรนเดอร์ PNG ด้วย crate `qrcode` (มีอยู่แล้ว, ใช้แบบเดียวกับ og.rs)
- [ ] `PaymentProvider::DirectTransfer` variant หรือ helper แยก: คืน `{ provider_ref: "direct_{id}", qr_url: "/qr/{id}.png" }` — route `/qr/{id}.png` generate จาก DB (settings.promptpay_id + amount) cache ใน memory
- [ ] `POST /api/donate/{id}/slip` — multipart รูป (จำกัด 5MB, ตรวจ magic bytes PNG/JPEG ไม่เชื่อ MIME header, ชื่อไฟล์ random) → status `awaiting_review`, บันทึก `slip_path`/`slip_at`; rate limit ต่อ IP เหมือน donate_limit; ปฏิเสธถ้า donation ไม่ใช่ `pending` หรือหมดอายุ
- [ ] **ตรวจอัตโนมัติ (optional ผ่าน env)**: `SLIP_API_URL` + `SLIP_API_KEY` (สไตล์ SlipOK) → trait `SlipVerifier` ที่ implement แรกคือ `Manual` (ไม่ทำอะไร รอกดอนุมัติ) และ `AutoApi` (ส่งรูป/refs ไปตรวจ, เทียบยอด+บัญชีปลายทาง, tolerance ยอด ±0, เวลา ±1 ชม.) — ถ้าเชื่อมไม่ได้/ไม่ตั้ง env = กลับเป็น manual ปลอดภัย
- [ ] Dashboard: `GET /api/me/slips?status=awaiting_review` (คิวรอตรวจ), `POST /api/me/slips/{id}/approve` → `settle(ref, "paid")` (SSE เด้งทันทีเหมือนเดิม), `POST /api/me/slips/{id}/reject` (body เหตุผล)
- [ ] หมดอายุ: ตอน `get_donation` ถ้า `pending` และ `now > expires_at` → อัปเดตเป็น `expired` (lazy check ไม่ต้องมี background job — PayPage poll ทุก 1-2 วิอยู่แล้ว)
- [ ] สลิป = ข้อมูลส่วนบุคคล: เสิร์ฟจาก `/uploads/slips/` **ผ่าน handler ที่ตรวจเจ้าของ** (ห้าม ServeDir ตรง ๆ สำหรับโฟลเดอร์นี้) — `GET /api/me/slips/{id}/image`
- [ ] ยอดรวมทุกที่ (goal_raised, leaderboard, recent, wallet) นับเฉพาะ `paid` เหมือนเดิม — อัตโนมัติเพราะ query เดิม condition `status='paid'` อยู่แล้ว

### 7.4 Frontend
- [ ] `Dashboard.vue` → settings: ส่วน "บัญชีรับเงิน (โอนตรง)" — promptpay_id + ธนาคารสำรอง, validate 10/13 หลักตัวเลข, แสดง QR ตัวอย่างทดสอบ ฿20
- [ ] `Dashboard.vue` → แท็บใหม่ **"คิวสลิป"**: รายการรอตรวจ (รูปสลิปคลิกขยาย, ยอด, ชื่อ, ข้อความ) + ปุ่มอนุมัติ/ปฏิเสธ + badge สถานะ; แท็บมี count แดงเมื่อมีรายการรอ
- [ ] `PayPage.vue`: ถ้า `pay_url` ว่างและ `qr_url` เป็น direct → แสดง QR จริง + ขั้นตอนใหม่: (1) สแกนจ่าย (2) **ปุ่ม "แนบสลิป"** (input file + preview) (3) สถานะ "รอตรวจสอบ…" → polling `GET /api/donate/{id}` จนได้ `paid` (confetti + เสียงเหมือนเดิม) หรือ `rejected` (แสดงเหตุผล)
- [ ] `DonatePage.vue`: badge "✓ ระบบตรวจสลิปอัตโนมัติ" ใต้ช่องโดเนต (สร้างความเชื่อมั่น)

### 7.5 งานเสริม
- [ ] `seed.rs`: เพิ่ม promptpay_id ตัวอย่างให้ streamer ทดสอบ
- [ ] `.env.example` + README: `SLIP_API_URL`, `SLIP_API_KEY`, วิธีทำงานของคิวสลิป
- [ ] Cargo.toml: ไม่ต้องเพิ่ม crate ใหม่ (qrcode/image/reqwest มีแล้ว)

### 7.6 การทดสอบยอมรับ
1. Mock mode (ไม่ตั้งค่าอะไร): โฟลว์เดิมไม่เปลี่ยน — สแกน QR ปลอม จ่าย mock แล้วเด้ง
2. DirectTransfer: ตั้ง promptpay_id → QR เป็น PromptPay จริง (สแกนด้วยแอปธนาคารเจอเบอร์+ยอดถูก) → แนบสลิป → คิวของเจ้าของเด้ง → อนุมัติ → overlay เด้ง + goal/leaderboard นับ
3. ปฏิเสธ → PayPage แสดงเหตุผล; หมดอายุ 15 นาที → สถานะ expired
4. คนอื่นเข้า `/api/me/slips` ไม่ได้ (403), สลิปผู้อื่นเปิดไม่ได้, อัปโหลดไฟล์ไม่ใช่รูปไม่ผ่าน
5. อัปโหลดสลิปซ้ำต่อ donation เดิม = แทนที่/ปฏิเสธ (เลือก: ปฏิเสธ ง่ายกว่า)

**ประมาณการ: ใหญ่สุดในแผน — ~60-70% ของงานทั้งหมด**

---

## Phase 8 (P1a) — สถานะวิดเจ็ตออนไลน์ + แตก Overlay เป็นหลายวิดเจ็ต

### 8.1 สถานะวิดเจ็ต (widget online)
- [ ] `AppState` เพิ่ม `overlay_conns: Arc<RwLock<HashMap<String, ConnInfo>>>` (`ConnInfo { count, last_seen }`)
- [ ] handler `/events`: inc ตอนเริ่ม, dec เมื่อ stream ปิด (Guard struct ที่ impl Drop ใน stream) + อัปเดต `last_seen`
- [ ] `GET /api/me/widget-status` → `{ online: bool, connections: u32, last_seen: Option<i64> }`
- [ ] `Dashboard.vue` overview: การ์ดใหม่บนสุด — "🟢 Overlay ออนไลน์ · 1 การเชื่อมต่อ" / "🔴 ยังไม่เชื่อมต่อ — เปิด overlay ใน OBS" (poll ทุก 15 วิ) + ปุ่มคัดลอก URL

### 8.2 แตก overlay เป็นวิดเจ็ตย่อย
- [ ] โครง `web/src/overlay/`: แยกเป็น components — `AlertWidget.vue` (ป็อบอัพ+เสียง), `GoalWidget.vue` (หลอดเป้า), `RecentWidget.vue` (feed ล่าสุด), `LeaderboardWidget.vue` (เท่าที่มีอยู่ + ย้ายมาเป็น widget เต็มตัว), `TopWidget.vue` (Top 3 แบบ EasyDonate)
- [ ] `overlay.html` + `?w=` — `w=all` (default, เหมือนเดิม ไม่พัง OBS เดิม), `w=alert`, `w=goal`, `w=recent`, `w=top`, `w=leaderboard`; รับ `?pos=` สำหรับตำแหน่ง widget บนจอ (ซ้าย/กลาง/ขวา)
- [ ] Goal/Recent/Top อัปเดตสด: ฟัง SSE event เดียวกัน (`useDonationWatch` มีอยู่แล้ว) → refresh ยอด/รายการ
- [ ] Dashboard settings: ตาราง "วิดเจ็ตและ URL" — แต่ละแถว: ชื่อ, URL พร้อม token, ปุ่มคัดลอก, สถานะออนไลน์ต่อ URL (จาก 8.1 แยกตาม `w` — เก็บ conn key เป็น `{user_id}:{widget}`)

**การทดสอบ**: เปิด overlay `w=alert` กับ `w=goal` คนละแท็บ → โดเนต → alert เด้งเฉพาะแท็บ alert, goal วิ่งทุกแท็บ; dashboard แสดงออนไลน์ถูกต้อง; `w=all` ยังทำงานเหมือนเดิม

**ประมาณการ: ~15%**

---

## Phase 9 (P1b) — ธีมหน้าโดเนต + ปก + About + โซเชียล + Live Preview

### 9.1 Migration `0008_page_custom.sql`
```sql
ALTER TABLE settings ADD COLUMN page_theme  TEXT NOT NULL DEFAULT 'rose';   -- rose | mint | midnight | retro
ALTER TABLE settings ADD COLUMN cover_url   TEXT NOT NULL DEFAULT '';       -- https:// only
ALTER TABLE settings ADD COLUMN about_text  TEXT NOT NULL DEFAULT '';       -- ≤ 400 chars
ALTER TABLE settings ADD COLUMN social_facebook TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_youtube  TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_twitch  TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_tiktok   TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_x        TEXT NOT NULL DEFAULT '';
```
(เก็บเป็น URL เต็ม https:// ทั้งหมด, validate ที่ update_settings — pattern เดียวกับ alert_image_url)

### 9.2 งาน
- [ ] `users.rs`: Settings + UpdateSettings + validate (ธีม whitelist, cover https, about ≤400, social https แต่ละอัน)
- [ ] `PublicProfile` เพิ่มฟิลด์ใหม่ทั้งหมด (หน้าโดเนตต้องใช้)
- [ ] `DonatePage.vue`:
  - ปก: ถ้ามี `cover_url` → hero เป็นรูปปก (object-fit cover, gradient overlay ให้ตัวหนังสืออ่านได้) แทน gradient เดิม
  - About: กล่อง "เกี่ยวกับฉัน" ใต้ hero (ถ้ากรอก)
  - โซเชียล: แถวไอคอน SVG inline (FB/YT/Twitch/TikTok/X) ลิงก์เปิดแท็บใหม่ `rel="noopener"`
  - ธีม: `page_theme` → `data-page-theme` attribute บน root → CSS variables (สี primary/พื้น/การ์ด) — 4 ธีม: **rose** (Stitch ปัจจุบัน), **mint**, **midnight**, **retro** (Win98-style แนวEasyDonate แต่ไม่ก็อป)
- [ ] `Dashboard.vue` → แท็บ settings ส่วน "หน้าโดเนตของฉัน": เลือกธีม (grid  thumbnail 4 ช่อง), ใส่ cover/about/social, **preview สด** = iframe `/#/u/{username}?preview=1` ที่ scale ลง (DonatePage โหมด preview ซ่อนฟอร์มโดเนต แสดงแค่หน้าตา)
- [ ] `og.rs`: ถ้ามี `cover_url` ใช้เป็น og:image แทน PNG generated (สวยกว่า)

**การทดสอบ**: เปลี่ยนธีม/ใส่ปก → preview อัปเดตทันที; หน้าจริงเปลี่ยนตาม; ลิงก์แชร์ LINE ได้ภาพปก; URL http:// ถูกปฏิเสธ; หน้าสตรีมเมอร์เก่า (ไม่ได้ตั้งค่า) = หน้าตาเดิมทุกประการ

**ประมาณการ: ~15%**

---

## Phase 10 (P2 — optional, ทำเมื่อ "ใช้จริง" เสถียรแล้ว)

ทำทีละข้อ แยก commit ได้ ไม่ผูกกัน:

### 10.1 แพลนรายเดือน + โควตาขึ้นจอ (โมเดลรายได้ของเรา)
- `0009_plans.sql`: `users.plan TEXT DEFAULT 'free'`, `plans` table (free: 20 alerts/เดือน, pro ฿49: ไม่จำกัด — เลียนแนวคิด EasyDonate แต่เหลือ 2 แพลน)
- นับที่ `settle()` จริง (ไม่ใช่ตอนสร้าง): เกินโควตา → donation สำเร็จแต่ alert ไม่เด้ง + dashboard แจ้งเตือน "อัปเกรด"
- หน้า billing mock (ยังไม่ตัดบัตรจริง) — ทำทีหลังเชื่อม payment จริง

### 10.2 รายงานภาษีรายปี
- `GET /api/me/tax-summary?year=2026` → ยอดรวม/จำนวนรายการ แยกรายเดือน (เฉพาะ paid)
- Dashboard → แท็บรายงาน: ตาราง 12 เดือน + ปุ่ม CSV รายปี (reuse export_csv) + ปุ่ม "พิมพ์" (print CSS แทน PDF generator — ไม่เพิ่ม dependency)

### 10.3 Webhook ออกด้านนอก (นักพัฒนา)
- settings: `webhook_url`, `webhook_secret` (แสดง secret ครั้งเดียว)
- หลัง settle paid → POST JSON event + ลายเซ็น HMAC-SHA256 header `X-DonateMe-Signature`, retry 3 ครั้ง
- หน้า log การส่ง 10 ครั้งล่าสุด (ตารางใน memory/DB)

### 10.4 i18n TH/EN
- vue-i18n + ไฟล์ `locales/th.ts`, `locales/en.ts`; toggle ใน SiteTopbar; เก็บ localStorage
- เริ่มจากหน้าสาธารณะ (Donate/Pay/Streamers) ก่อน — dashboard ไทยอย่างเดียวได้

---

## ลำดับการทำงาน & ระยะเวลาโดยประมาณ

| ลำดับ | Phase | ประมาณ | ผลลัพธ์ที่วัดได้ |
|---|---|---|---|
| 1 | **Phase 7** โอนตรง+สลิป | ใหญ่สุด (~60-70%) | รับเงินจริงได้ 0% ค่าธรรมเนียม |
| 2 | **Phase 8** widget status + แตก overlay | ~15% | dashboard รู้ว่า OBS ออนไลน์, OBS วาง widget แยกได้ |
| 3 | **Phase 9** ธีม+ปก+about+โซเชียล | ~15% | สตรีมเมอร์แต่งหน้าโดเนตเป็นอัตลักษณ์ |
| 4 | Phase 10 (ทยอย) | ตาม需求 | โควตา/ภาษี/webhook/i18n |

แต่ละ phase: commit แยก + อัปเดต README + ทดสอบ browser จริง (เหมือนที่ทำมา) ก่อนขึ้น phase ใหม่

## ความเสี่ยง & ทางออก

| ความเสี่ยง | ทางออก |
|---|---|
| API ตรวจสลิปภายนอกมีค่าใช้จ่าย/ไม่เสถียร | ออกแบบเป็น `SlipVerifier` trait — manual approve คือ default ที่ใช้ได้เสมอ |
| ผู้ชมไม่แนบสลิป (โอนแล้วเงียบ) | สตรีมเมอร์เห็น "รอโอน" หมดอายุ 15 นาที; อนาคตเพิ่ม "กดยืนยันเอง" สำหรับยอดเล็ก |
| QR PromptPay ผิด spec สแกนไม่ได้ | เทสกับแอปธนาคารจริงหลายค่ายใน Phase 7.6 + CRC16 เขียน unit test (เคสสำเร็จจาก spec GB/T 18284) |
| สลิปปลอม | manual approve เป็นตัวชี้ขาด; auto API เทียบเลขอ้างอิง+ยอด+เวลา; แสดงชื่อบัญชีผู้โอนจาก API เมื่อมี |
| อัปโหลดรูปอันตราย | magic bytes + ขนาดจำกัด + เสิร์ฟด้วย Content-Type image/* บังคับ + ชื่อไฟล์ random + auth เจ้าของ |
