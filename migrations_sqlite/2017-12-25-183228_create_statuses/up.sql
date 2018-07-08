CREATE TABLE statuses (
    id BIGINT PRIMARY KEY NOT NULL,
    text TEXT NOT NULL,
    content_warning TEXT,

    created_at DATETIME NOT NULL,

    account_id BIGINT REFERENCES accounts(id) NOT NULL
);
