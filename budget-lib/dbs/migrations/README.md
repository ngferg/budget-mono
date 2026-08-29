# User database migrations

Each user has their own SQLite database at `$SQLITE_DB_PATH/<hashed_email>.db`,
created from [`../USER_DDL.sql`](../USER_DDL.sql) when the account is made. New
databases are always at the latest schema; the files here bring **existing**
databases up to date.

## Running

```sh
make migrate-dbs                      # uses SQLITE_DB_PATH=budget-lib/dbs
./budget-lib/dbs/migrations/apply.sh  # same, script self-locates the dir
./budget-lib/dbs/migrations/apply.sh /path/to/prod/dbs
```

`apply.sh` needs the `sqlite3` CLI. It records applied migrations in each
database's `schema_migrations` table, so it is safe to run repeatedly and safe
to run after new accounts have been created.

## Conventions

- One `NNNN_description.sql` file per change, applied in filename order.
- Every migration must be idempotent on its own (`CREATE TABLE IF NOT EXISTS`,
  `INSERT OR IGNORE`, guarded `UPDATE`s) as a second line of defence.
- When a migration changes the schema of a fresh database, make the same change
  in `../USER_DDL.sql` **and** add the filename to the `INSERT INTO
  schema_migrations` list there, so new databases are marked as already having
  it.

## Current migrations

| File | Purpose |
| ---- | ------- |
| `0001_add_subscription.sql` | Adds the `subscription` table to pre-monetization databases. |
| `0002_grandfather_existing_users.sql` | Grants `lifetime` status to every account that existed before monetization launched. |
| `0003_add_subscription_cancellation.sql` | Adds the `cancel_at_period_end` column. Not self-idempotent (SQLite has no conditional `ADD COLUMN`); relies on the runner's tracking. |

## Subscription model

The `subscription` table holds one row (`id = 1`).

| `status` | Meaning |
| -------- | ------- |
| `free_trial` | One free month from account creation (`trial_started_at` … `trial_ends_at`). |
| `active` | Paying $5/month via Stripe. `current_period_end` mirrors the paid-through date. |
| `inactive` | Trial ended, or a paid subscription lapsed, with nothing active. |
| `lifetime` | One-time $100 lifetime license, or the early-adopter grant. |

`cancel_at_period_end = 1` marks an `active` row whose Stripe subscription is set
to stop renewing. Access continues until `current_period_end`; after that the
`customer.subscription.deleted` webhook moves `status` to `inactive`. A fresh
Checkout resets the flag to `0`.

The $5/month plan lives in Stripe as a Product + recurring Price, created once
with `make create-stripe-product` (see the repo README). The `stripe_*` columns
are filled in by the `/webhooks/stripe` handler when a Checkout Session
completes; they are `NULL` until the user subscribes.
