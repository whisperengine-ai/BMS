# BMS – AI Agent Working Notes

Use this as your ops manual for quick, correct changes. Keep edits small, deterministic, and aligned with the telic-compression design.

## Big picture
- Workspace crates:
  - `bms-core`: canonical JSON, coordinates, deltas (RFC 6902), Merkle, snapshots.
  - `bms-storage`: SQLite via sqlx; repository pattern. No raw SQL outside `repository.rs`.
  - `bms-vector`: embeddings via FastEmbed; simple in-memory vector store for CLI-only flows.
  - `bms-api`: Axum REST; on-demand vector search; no persistent vectors.
  - `bms-cli`: developer CLI; local search fallback builds an in-memory index.
- Canonical data = deltas + snapshots only. Vectors are search metadata computed at query time. See `docs/BMS_DESIGN.txt` §5.3 and `docs/VECTOR_SEARCH_DESIGN.md`.

## Where to make changes
- API endpoint logic: `crates/bms-api/src/handlers.rs` (add handler, request/response types with serde).
- API routing and state: `crates/bms-api/src/main.rs` and `state.rs`.
- Core primitives or algorithms: `crates/bms-core/src/*.rs`.
- Storage ops (only place to touch SQL): `crates/bms-storage/src/repository.rs` (+ models/schema).
- CLI commands: `crates/bms-cli/src/main.rs`.
- Embeddings/vec search utilities: `crates/bms-vector/src/*` (default model: all-MiniLM-L6-v2, 384 dims).

## Critical patterns and conventions
- Determinism first: use canonical JSON for hashing and coordinate generation (`bms-core`).
- IDs and hashes:
  - `CoordId` = 128-bit addr, Base32 ASCII (26 chars typical).
  - `DeltaId` = sha3-256 of canonical delta (first 16 bytes hex).
  - Snapshot policy: `DEFAULT_SNAPSHOT_INTERVAL` (128) unless overridden.
- Reconstruction: prefer `SnapshotManager::reconstruct(snapshot, &deltas[..])`; else replay deltas in order with `DeltaEngine::apply_delta`.
- API search is on-demand:
  - Don’t write embeddings on `/store`.
  - `/search` reconstructs head states, computes `head_hash = sha3(state_json)`, caches embeddings by `CoordId` when the head hash matches.
  - Cosine similarity ranking in-memory; optional `min_score`, `limit`, and simple author filter.
  - See `bms-api/src/handlers.rs::search` and `state.rs::CachedEmbedding`.
- Logging: use `tracing`; keep messages concise, include coord IDs when helpful.
- Errors: map to `AppError` in API; return structured JSON with appropriate HTTP status.
- Storage access: go through `BmsRepository`; add new queries there with typed rows in `models.rs` and SQL in `schema.rs` if needed.

## Developer workflows (commands)
- Build all: `cargo build --release`
- Test all: `cargo test --workspace`
- Run CLI: `cargo run --bin bms -- --db-path demo.db list`
- Run API (port 3000): `cargo run --bin bms-api` (optional `BMS_DB_PATH=./bms.db`)
- Demo end-to-end: `./demo.sh` (init → store → recall → verify → list → stats → search)
- Env vars: `BMS_DB_PATH`, `RUST_LOG`, optional `BMS_API_URL` for CLI to hit API.

## Integration details to remember
- Embeddings: `bms-vector::EmbeddingGenerator` (FastEmbed). Default dim = 384. Don’t persist vectors in POC.
- CLI local search: builds an in-memory index using `InMemoryVectorStore`; API path uses the on-demand cache in `AppState` instead.
- SQLite: single-writer; all access via async `sqlx` in `BmsRepository`.

## Examples from the codebase
- Add an API route: register in `bms-api/src/main.rs`, implement handler in `handlers.rs`, define request/response with `serde`, and use `BmsRepository` for DB ops.
- Search flow reference: `handlers.rs::search` shows reconstruct → embed-or-cache → cosine → filter/sort → top-k.
- Snapshot creation: `handlers.rs::create_snapshot` and `SnapshotManager::create_snapshot`.

## Gotchas
- Cache correctness: always recompute `head_hash` from the reconstructed head and invalidate cache on mismatch.
- Don’t introduce vector writes in `/store`; keep vectors ephemeral per design docs.
- Keep embedding dimension consistent with the model (384); validate lengths in stores/search.

## Key docs
- `README.md` (API usage and quick start)
- `docs/BMS_DESIGN.txt` (authoritative design brief)
- `docs/VECTOR_SEARCH_DESIGN.md` (on-demand vector indexing)
- `DEVELOPMENT.md` (dev workflows; note some vector notes are pre-refactor)

If something’s ambiguous, prefer aligning with `docs/BMS_DESIGN.txt` and the existing patterns in `handlers.rs` and `repository.rs`. Keep changes small and test with `cargo test` and `./demo.sh`. 