CREATE TABLE IF NOT EXISTS observations (
    id UUID PRIMARY KEY,
    content_hash TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    title TEXT,
    source_url TEXT,
    source_kind TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS chunks (
    id UUID PRIMARY KEY,
    observation_id UUID NOT NULL REFERENCES observations (id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    text TEXT NOT NULL,
    start_offset BIGINT NOT NULL,
    end_offset BIGINT NOT NULL,
    token_estimate INTEGER NOT NULL,
    UNIQUE (observation_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS chunks_observation_id_idx ON chunks (observation_id);
