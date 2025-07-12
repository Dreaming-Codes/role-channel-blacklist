-- Drop indexes first
DROP INDEX IF EXISTS idx_exception_channel_role;
DROP INDEX IF EXISTS idx_exception_role_id;
DROP INDEX IF EXISTS idx_exception_channel_id;

DROP INDEX IF EXISTS idx_blacklist_channel_role;
DROP INDEX IF EXISTS idx_blacklist_role_id;
DROP INDEX IF EXISTS idx_blacklist_channel_id;

-- Drop tables
DROP TABLE IF EXISTS exception_entries;
DROP TABLE IF EXISTS blacklist_entries;
