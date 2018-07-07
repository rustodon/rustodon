UPDATE accounts SET domain = NULL WHERE domain = '';
DROP INDEX accounts_must_have_unique_uris;
