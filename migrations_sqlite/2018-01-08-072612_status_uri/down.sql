-- TODO: You can't ALTER TABLE DROP COLUMN in SQLite, so this'll have to be
-- a command to reinstate the old table schema and reload it
ALTER TABLE statuses DROP COLUMN uri RESTRICT;

