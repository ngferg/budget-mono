CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category TEXT,
  is_expense BOOLEAN
);

INSERT INTO categories
  (category, is_expense)
VALUES
  ('Income', 0),
  ('House', 1),
  ('Utilities', 1),
  ('Transportation', 1),
  ('Food', 1),
  ('Entertainment', 1),
  ('Health', 1),
  ('Charity', 1);

CREATE TABLE IF NOT EXISTS budget (
    year INTEGER,
    month INTEGER,
    PRIMARY KEY (year, month)
);

CREATE TABLE IF NOT EXISTS line_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  description TEXT,
  amount INTEGER,
  category INTEGER,
  budget_year INTEGER,
  budget_month INTEGER,
  FOREIGN KEY(category) REFERENCES categories(id),
  FOREIGN KEY(budget_year, budget_month) REFERENCES budget(year, month)
);

INSERT INTO budget (year, month) VALUES (STRFTIME('%Y','now'), STRFTIME('%m','now'));

WITH vars AS (
  SELECT STRFTIME('%Y','now') AS current_year,
         STRFTIME('%m','now') AS current_month
)
INSERT INTO line_items (amount, description, category, budget_year, budget_month)
SELECT 100000, 'Rent', 2, current_year, current_month FROM vars
UNION ALL SELECT 15000, 'Electric', 3, current_year, current_month FROM vars
UNION ALL SELECT 200000, 'First paycheck', 1, current_year, current_month FROM vars
UNION ALL SELECT 200000, 'Second paycheck', 1, current_year, current_month FROM vars;

-- ---------------------------------------------------------------------------
-- Monetization
-- ---------------------------------------------------------------------------

-- Single row (id is pinned to 1) describing this user's subscription state.
--
--   status = 'free_trial'  one free month, starting when the account is created
--   status = 'active'      paying $5/month through Stripe
--   status = 'inactive'    trial ended (or a paid subscription lapsed) without
--                          an active subscription
--   status = 'lifetime'    one-time $100 lifetime license (also granted to every
--                          account that predates monetization -- see
--                          dbs/migrations/0002_grandfather_existing_users.sql)
--
-- The stripe_* columns stay NULL until payments are wired up; they are here so
-- the webhook handler has somewhere to record Stripe's identifiers.
CREATE TABLE IF NOT EXISTS subscription (
  id                       INTEGER PRIMARY KEY CHECK (id = 1),
  status                   TEXT NOT NULL DEFAULT 'free_trial'
                             CHECK (status IN ('free_trial', 'active', 'inactive', 'lifetime')),
  trial_started_at         TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  trial_ends_at            TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now', '+1 month')),
  -- For an active monthly subscription: the end of the paid-through period,
  -- mirrored from Stripe so access can be checked without calling their API.
  current_period_end       TEXT,
  -- Set to 1 when the user cancels: the subscription stops renewing but access
  -- continues until current_period_end, after which the customer.subscription
  -- .deleted webhook moves status to 'inactive'.
  cancel_at_period_end     INTEGER NOT NULL DEFAULT 0,
  stripe_customer_id       TEXT,
  stripe_subscription_id   TEXT,
  stripe_payment_intent_id TEXT,
  updated_at               TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT INTO subscription (id) VALUES (1);

-- Records which files from dbs/migrations/ have been applied to this database.
-- A database created from this DDL is already at the latest schema, so the
-- current migrations are recorded as applied to keep the runner idempotent and
-- to keep new accounts out of the early-adopter grant.
CREATE TABLE IF NOT EXISTS schema_migrations (
  name       TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT INTO schema_migrations (name) VALUES
  ('0001_add_subscription.sql'),
  ('0002_grandfather_existing_users.sql'),
  ('0003_add_subscription_cancellation.sql');
