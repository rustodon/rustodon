CREATE TABLE statuses (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    content_warning TEXT,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL,

    account_id BIGINT REFERENCES accounts(id) NOT NULL
);
