CREATE TABLE IF NOT EXISTS subscriptions (
    room_id TEXT PRIMARY KEY NOT NULL,
    room_name TEXT NOT NULL,
    meter_id TEXT NOT NULL,
    subscribed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daily_usage_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL REFERENCES subscriptions(room_id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    usage REAL NOT NULL,
    cost REAL NOT NULL,
    UNIQUE(room_id, date)
);

CREATE INDEX IF NOT EXISTS ix_daily_usage_room_date
    ON daily_usage_records(room_id, date);

CREATE TABLE IF NOT EXISTS daily_balance_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL REFERENCES subscriptions(room_id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    balance REAL NOT NULL,
    UNIQUE(room_id, date)
);

CREATE INDEX IF NOT EXISTS ix_daily_balance_room_date
    ON daily_balance_records(room_id, date);

CREATE TABLE IF NOT EXISTS backfill_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL UNIQUE REFERENCES subscriptions(room_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    started_at TEXT,
    updated_at TEXT NOT NULL,
    error TEXT
);
