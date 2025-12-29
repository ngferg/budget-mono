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
SELECT 100000, 'Rent', 3, current_year, current_month FROM vars
UNION ALL SELECT 15000, 'Electric', 2, current_year, current_month FROM vars
UNION ALL SELECT 200000, 'First paycheck', 1, current_year, current_month FROM vars
UNION ALL SELECT 200000, 'Second paycheck', 1, current_year, current_month FROM vars;
