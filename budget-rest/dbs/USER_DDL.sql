CREATE TABLE IF NOT EXISTS categories (
  id INT,  
  category VARCHAR(31),
  is_expense BOOLEAN,
  PRIMARY KEY (id)
);

INSERT INTO categories
  (id, category, is_expense)
VALUES
  (0, "Income", 0),
  (1, "House", 1),
  (2, "Utilities", 1),
  (3, "Transporttion", 1),
  (4, "Charity", 1);


CREATE TABLE IF NOT EXISTS budget (
    year INT,
    month INT,
    PRIMARY KEY (year, month)
);
