/// SQL schema for BMS storage
pub const SCHEMA_SQL: &str = r#"
-- Coordinates table
CREATE TABLE IF NOT EXISTS coordinates (
    id_ascii TEXT PRIMARY KEY NOT NULL,
    rune_alias TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_coords_created ON coordinates(created_at);

-- Deltas table
CREATE TABLE IF NOT EXISTS deltas (
    id TEXT PRIMARY KEY NOT NULL,
    coord_id TEXT NOT NULL,
    parent_id TEXT,
    parent_hash TEXT,
    delta_hash TEXT NOT NULL,
    chain_hash TEXT NOT NULL,
    ops TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tags TEXT,
    author TEXT,
    FOREIGN KEY (coord_id) REFERENCES coordinates(id_ascii) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_deltas_coord ON deltas(coord_id, created_at);
CREATE INDEX IF NOT EXISTS idx_deltas_parent ON deltas(parent_id);
CREATE INDEX IF NOT EXISTS idx_deltas_created ON deltas(created_at);

-- Snapshots table
CREATE TABLE IF NOT EXISTS snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    coord_id TEXT NOT NULL,
    head_delta_id TEXT NOT NULL,
    state_hash TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (coord_id) REFERENCES coordinates(id_ascii) ON DELETE CASCADE,
    FOREIGN KEY (head_delta_id) REFERENCES deltas(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_snapshots_coord ON snapshots(coord_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_hash ON snapshots(state_hash);

-- Metadata table for system info
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO metadata (key, value) VALUES ('created_at', datetime('now'));
"#;
