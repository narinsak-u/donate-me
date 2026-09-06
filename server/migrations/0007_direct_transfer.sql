-- Phase 7: โอนตรง + ยืนยันสลิป (DirectTransfer)
-- เงินโอนเข้าบัญชีสตรีมเมอร์โดยตรง ไม่ผ่านแพลตฟอร์ม
ALTER TABLE settings ADD COLUMN promptpay_id TEXT NOT NULL DEFAULT '';   -- เบอร์โทร 10 หลัก หรือบัตร 13 หลัก
ALTER TABLE settings ADD COLUMN bank_name TEXT NOT NULL DEFAULT '';      -- ช่องทางสำรอง (แสดงบน PayPage)
ALTER TABLE settings ADD COLUMN bank_no TEXT NOT NULL DEFAULT '';

ALTER TABLE donations ADD COLUMN slip_path TEXT NOT NULL DEFAULT '';     -- slips/{random}.png (เสิร์ฟผ่าน auth เท่านั้น)
ALTER TABLE donations ADD COLUMN slip_at INTEGER;                        -- เวลาแนบสลิป
ALTER TABLE donations ADD COLUMN reviewed_at INTEGER;                    -- เวลาอนุมัติ/ปฏิเสธ
ALTER TABLE donations ADD COLUMN review_note TEXT NOT NULL DEFAULT '';   -- เหตุผลตอนปฏิเสธ
ALTER TABLE donations ADD COLUMN expires_at INTEGER;                     -- deadline โอน (created_at + 15 นาที)
