CREATE TABLE statuses (
    id BIGINT PRIMARY KEY,
    text TEXT NOT NULL,
    content_warning TEXT,

    created_at DATETIME NOT NULL,

    account_id BIGINT REFERENCES accounts(id) NOT NULL
);
