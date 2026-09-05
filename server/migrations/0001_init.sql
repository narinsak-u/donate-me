-- Phase 1: ตารางหลักของระบบโดเนต (SQLite สำหรับ mock phase)
CREATE TABLE IF NOT EXISTS donations (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL DEFAULT 'default',
    donor_name   TEXT NOT NULL,
    amount       INTEGER NOT NULL CHECK (amount >= 1 AND amount <= 100000),
    message      TEXT NOT NULL DEFAULT '',
    sound        TEXT NOT NULL DEFAULT 'chime',
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'failed', 'expired')),
    payment_ref  TEXT NOT NULL UNIQUE,
    created_at   INTEGER NOT NULL,
    paid_at      INTEGER
);

CREATE INDEX IF NOT EXISTS idx_donations_user_status ON donations(user_id, status);
CREATE INDEX IF NOT EXISTS idx_donations_created ON donations(created_at DESC);
