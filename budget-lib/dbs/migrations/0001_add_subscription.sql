-- Adds the subscription table to user databases created before monetization.
-- Keep this definition in sync with dbs/USER_DDL.sql.
--
-- Idempotent: CREATE TABLE IF NOT EXISTS + INSERT OR IGNORE mean re-running is a
-- no-op, though the migration runner (apply.sh) also skips already-applied files.
CREATE TABLE IF NOT EXISTS subscription (
  id                       INTEGER PRIMARY KEY CHECK (id = 1),
  status                   TEXT NOT NULL DEFAULT 'free_trial'
                             CHECK (status IN ('free_trial', 'active', 'inactive', 'lifetime')),
  trial_started_at         TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  trial_ends_at            TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now', '+1 month')),
  current_period_end       TEXT,
  stripe_customer_id       TEXT,
  stripe_subscription_id   TEXT,
  stripe_payment_intent_id TEXT,
  updated_at               TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Existing accounts have already used far more than a month, so starting them on
-- a fresh trial would be wrong. 0002 moves these rows to 'lifetime'; this insert
-- just guarantees the row exists.
INSERT OR IGNORE INTO subscription (id) VALUES (1);
