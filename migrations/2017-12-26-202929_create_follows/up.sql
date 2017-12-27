CREATE TABLE follows (
    id BIGSERIAL PRIMARY KEY,
    source_id BIGINT REFERENCES accounts(id) NOT NULL,
    target_id BIGINT REFERENCES accounts(id) NOT NULL,


    UNIQUE(source_id, target_id)
);
