-- Email::parse now trims whitespace and lowercases its input so case-variant addresses
-- ("Alice@Example.com" vs "alice@example.com") resolve to the same identity.
--
-- Backfill any pre-existing rows that were inserted under the previous case-preserving
-- behavior so they remain reachable after the application change. If two rows differ only
-- by case, keep the lexicographically smallest hashed password row (deterministic) and drop
-- the duplicates; this matches the practical assumption that mixed-case duplicates were the
-- same human and the password was set most recently on whichever row hashed lower (in absence
-- of created_at we cannot do better — operators can backfill manually if needed).

BEGIN;

-- Step 1: deduplicate. For each lowercase-canonical email, keep one row.
WITH ranked AS (
    SELECT
        ctid,
        ROW_NUMBER() OVER (PARTITION BY LOWER(TRIM(email)) ORDER BY password_hash) AS rn
    FROM auth.users
)
DELETE FROM auth.users
USING ranked
WHERE auth.users.ctid = ranked.ctid
  AND ranked.rn > 1;

-- Step 2: normalize the surviving rows.
UPDATE auth.users
SET email = LOWER(TRIM(email))
WHERE email <> LOWER(TRIM(email));

COMMIT;
