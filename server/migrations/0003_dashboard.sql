-- Phase 3: dashboard — ซ่อนข้อความ + ไฟล์เสียง alert ของสตรีมเมอร์
ALTER TABLE donations ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE settings ADD COLUMN alert_sound_url TEXT NOT NULL DEFAULT '';
