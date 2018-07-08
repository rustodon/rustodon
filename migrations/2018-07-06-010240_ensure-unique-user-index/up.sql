UPDATE accounts SET domain = '' WHERE domain IS NULL;
CREATE UNIQUE INDEX accounts_must_have_unique_uris ON accounts(uri);
