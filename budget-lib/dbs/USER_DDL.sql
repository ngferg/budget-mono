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

