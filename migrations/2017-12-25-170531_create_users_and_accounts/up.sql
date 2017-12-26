CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,
    domain VARCHAR,

    username VARCHAR NOT NULL,
    display_name VARCHAR,
    summary TEXT
);

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR NOT NULL,
    encrypted_password VARCHAR NOT NULL,
    account_id BIGINT REFERENCES accounts(id) NOT NULL
);
