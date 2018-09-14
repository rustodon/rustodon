CREATE TYPE job_status AS ENUM ('waiting', 'running', 'dead');

CREATE TABLE jobs (
    id BIGINT PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,

    status job_status NOT NULL,

    kind VARCHAR NOT NULL,
    job_data JSONB NOT NULL
)
