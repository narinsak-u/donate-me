-- Phase 5: leaderboard + media alert
ALTER TABLE settings ADD COLUMN alert_image_url TEXT NOT NULL DEFAULT '';
-- แสดง leaderboard บนหน้าโดเนตสาธารณะ (0=ปิด, 1=เปิด)
ALTER TABLE settings ADD COLUMN show_leaderboard INTEGER NOT NULL DEFAULT 1;
