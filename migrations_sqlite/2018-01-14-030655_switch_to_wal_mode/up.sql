-- diesel_migrations runs each migration inside a transaction, and
-- there is currently no way to opt-out of this behavior.  For most
-- data definition tasks, this is fine.
--
-- However, the journal mode for an SQLite database cannot be set
-- within a transaction.  To get around this, we close the Diesel
-- transaction with COMMIT, set the journal mode, and then open
-- another transaction that Diesel can finish.  It's kinda gross
-- but ¯\_(ツ)_/¯

COMMIT;

PRAGMA journal_mode=WAL;

BEGIN TRANSACTION;
