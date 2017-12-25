CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    domain VARCHAR
);

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR NOT NULL,
    encrypted_password VARCHAR NOT NULL,
    account_id BIGINT REFERENCES accounts(id) NOT NULL
);
