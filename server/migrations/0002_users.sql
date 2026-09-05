-- Phase 2: ระบบสมาชิก + การตั้งค่าของสตรีมเมอร์แต่ละคน
CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE CHECK (username GLOB '[a-z0-9_]*' AND length(username) BETWEEN 3 AND 20),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name  TEXT NOT NULL,
    created_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    user_id             TEXT PRIMARY KEY REFERENCES users(id),
    theme               TEXT NOT NULL DEFAULT 'pink',
    goal_amount         INTEGER NOT NULL DEFAULT 0,
    alert_duration_sec  INTEGER NOT NULL DEFAULT 6,
    tts_enabled         INTEGER NOT NULL DEFAULT 1,
    alert_text          TEXT NOT NULL DEFAULT 'ขอบคุณ {name} ที่โดเนต {amount} บาท'
);
