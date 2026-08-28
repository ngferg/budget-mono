#!/bin/sh
# Applies every *.sql migration in this directory to each per-user SQLite
# database, exactly once per database.
#
# Usage:
#   ./apply.sh [DB_DIR]
#
# DB_DIR resolution order:
#   1. the first CLI argument
#   2. $SQLITE_DB_PATH   (the same variable the services use)
#   3. this script's parent directory (budget-lib/dbs)
#
# Every "<something>.db" file in DB_DIR is treated as a user database. Applied
# migrations are tracked in each database's schema_migrations table, so running
# this repeatedly is safe.
set -eu

MIGRATIONS_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DB_DIR=${1:-${SQLITE_DB_PATH:-$(dirname -- "$MIGRATIONS_DIR")}}

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "error: the sqlite3 CLI is required but was not found on PATH" >&2
  exit 1
fi

if [ ! -d "$DB_DIR" ]; then
  echo "error: db directory not found: $DB_DIR" >&2
  exit 1
fi

echo "Applying migrations from $MIGRATIONS_DIR"
echo "                     to databases in $DB_DIR"

found_db=0
for db in "$DB_DIR"/*.db; do
  [ -e "$db" ] || continue
  found_db=1
  echo "==> $(basename -- "$db")"

  sqlite3 -bail "$db" "CREATE TABLE IF NOT EXISTS schema_migrations (
    name       TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
  );"

  for migration in "$MIGRATIONS_DIR"/*.sql; do
    [ -e "$migration" ] || continue
    name=$(basename -- "$migration")

    applied=$(sqlite3 "$db" "SELECT 1 FROM schema_migrations WHERE name = '$name';")
    if [ "$applied" = "1" ]; then
      echo "    skip  $name"
      continue
    fi

    echo "    apply $name"
    # Run the migration and record it in a single transaction. -bail makes
    # sqlite3 exit non-zero on the first error, which (with set -e) aborts
    # before COMMIT so the transaction rolls back.
    {
      echo "BEGIN;"
      cat -- "$migration"
      echo "INSERT INTO schema_migrations (name) VALUES ('$name');"
      echo "COMMIT;"
    } | sqlite3 -bail "$db"
  done
done

if [ "$found_db" -eq 0 ]; then
  echo "No *.db files found in $DB_DIR -- nothing to do."
fi

echo "Done."
