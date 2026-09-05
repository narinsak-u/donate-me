-- Phase 7: กระเป๋าเงิน & ถอน (mock — ยังไม่มีการโอนจริง รอ Phase Omise Payout)
CREATE TABLE IF NOT EXISTS withdrawals (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL,
    amount       INTEGER NOT NULL CHECK (amount >= 100),
    bank_name    TEXT NOT NULL,
    bank_account TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'rejected')),
    created_at   INTEGER NOT NULL,
    paid_at      INTEGER
);

CREATE INDEX IF NOT EXISTS idx_withdrawals_user ON withdrawals(user_id, created_at DESC);
