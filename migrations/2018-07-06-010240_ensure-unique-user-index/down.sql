-- This file should undo anything in `up.sql`
UPDATE accounts SET domain = NULL WHERE domain = '';
DROP INDEX accounts_must_have_unique_uris;