-- Review fixes: ตำแหน่งป็อบอัพ, TTS settings, tier thresholds, pin ข้อความ
ALTER TABLE settings ADD COLUMN alert_position TEXT NOT NULL DEFAULT 'middle' CHECK (alert_position IN ('top','middle','bottom'));
ALTER TABLE settings ADD COLUMN tts_speed REAL NOT NULL DEFAULT 1.0;
ALTER TABLE settings ADD COLUMN tts_max_len INTEGER NOT NULL DEFAULT 120;
ALTER TABLE settings ADD COLUMN tier_vip_amount INTEGER NOT NULL DEFAULT 100;
ALTER TABLE settings ADD COLUMN tier_gold_amount INTEGER NOT NULL DEFAULT 500;
ALTER TABLE donations ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
