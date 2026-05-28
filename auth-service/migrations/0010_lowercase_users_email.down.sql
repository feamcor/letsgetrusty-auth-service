-- The 'up' migration is non-reversible: lowercasing destroys original case information, and
-- the deduplication step deletes rows we cannot recover. The down migration is a no-op.
SELECT 1;
