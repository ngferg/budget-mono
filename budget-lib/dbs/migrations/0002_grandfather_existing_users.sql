-- Early-adopter grant: every account that already existed when monetization
-- launched gets lifetime access for free.
--
-- This file is only ever applied to databases that predate the subscription
-- schema in USER_DDL.sql. Databases created from the current DDL record both
-- 0001 and 0002 as already-applied, so apply.sh never runs this against a
-- genuinely new account. That makes the blanket UPDATE safe.
UPDATE subscription
SET status     = 'lifetime',
    updated_at = STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE id = 1;
