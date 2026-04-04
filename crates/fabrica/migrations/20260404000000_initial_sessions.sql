-- Initial schema: sessions table for per-project databases
CREATE TABLE IF NOT EXISTS sessions (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    layout_state TEXT,
    git_head     TEXT
);
