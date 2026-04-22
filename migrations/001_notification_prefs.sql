-- Global channel defaults per user
CREATE TABLE IF NOT EXISTS notification_channel_defaults (
    user_id     TEXT NOT NULL,
    channel     TEXT NOT NULL,  -- 'email' | 'push' | 'sms'
    enabled     INTEGER NOT NULL DEFAULT 1,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, channel)
);

-- Per-event-type overrides per user
CREATE TABLE IF NOT EXISTS notification_event_prefs (
    user_id     TEXT NOT NULL,
    event_type  TEXT NOT NULL,  -- e.g. 'order.shipped'
    channel     TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, event_type, channel)
);
