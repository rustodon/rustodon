CREATE TYPE job_status AS ENUM ('waiting', 'running', 'dead');

CREATE TABLE jobs (
    id BIGINT PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,

    status job_status NOT NULL,

    queue VARCHAR NOT NULL,
    kind VARCHAR NOT NULL,
    data JSONB NOT NULL
)
