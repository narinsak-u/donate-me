-- Phase 7 (ต่อ): CHECK เดิมล็อก status ไว้ 4 ค่า — SQLite แก้ constraint ไม่ได้ ต้อง rebuild ตาราง
-- สถานะใหม่: awaiting_review (แนบสลิปแล้วรอเจ้าของตรวจ), rejected (ปฏิเสธสลิป)
CREATE TABLE donations_new (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL DEFAULT 'default',
    donor_name   TEXT NOT NULL,
    amount       INTEGER NOT NULL CHECK (amount >= 1 AND amount <= 100000),
    message      TEXT NOT NULL DEFAULT '',
    sound        TEXT NOT NULL DEFAULT 'chime',
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'awaiting_review', 'paid', 'failed', 'expired', 'rejected')),
    payment_ref  TEXT NOT NULL UNIQUE,
    created_at   INTEGER NOT NULL,
    paid_at      INTEGER,
    hidden       INTEGER NOT NULL DEFAULT 0,
    pinned       INTEGER NOT NULL DEFAULT 0,
    slip_path    TEXT NOT NULL DEFAULT '',
    slip_at      INTEGER,
    reviewed_at  INTEGER,
    review_note  TEXT NOT NULL DEFAULT '',
    expires_at   INTEGER
);

INSERT INTO donations_new
    (id, user_id, donor_name, amount, message, sound, status, payment_ref, created_at, paid_at, hidden, pinned, slip_path, slip_at, reviewed_at, review_note, expires_at)
SELECT
    id, user_id, donor_name, amount, message, sound, status, payment_ref, created_at, paid_at, hidden, pinned, slip_path, slip_at, reviewed_at, review_note, expires_at
FROM donations;

DROP TABLE donations;
ALTER TABLE donations_new RENAME TO donations;

CREATE INDEX IF NOT EXISTS idx_donations_user_status ON donations(user_id, status);
CREATE INDEX IF NOT EXISTS idx_donations_created ON donations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_donations_slip_queue ON donations(user_id, status, slip_at);
