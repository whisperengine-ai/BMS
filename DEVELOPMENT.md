# BMS Development Guide

## Project Status: POC/MVP Phase 1 ✅

### Completed Features

#### Core Engine (`bms-core`)
- ✅ Canonical JSON serialization with deterministic ordering
- ✅ Coordinate generation (128-bit SHA3-256, base32 encoding)
- ✅ Delta compression using RFC 6902 JSON Patch
- ✅ Merkle chain verification for tamper-evidence
- ✅ Snapshot management with configurable intervals
- ✅ 24 passing unit tests

#### Storage Layer (`bms-storage`)
- ✅ SQLite-based KV store with async support (sqlx)
- ✅ Tables: coordinates, deltas, snapshots, metadata
- ✅ Optimized indexes for query performance
- ✅ Repository pattern with full CRUD operations

#### API Server (`bms-api`)
- ✅ Axum-based REST API
- ✅ Endpoints: `/store`, `/recall`, `/verify`, `/snapshot`, `/coords`, `/stats`
- ✅ Async request handling
- ✅ Structured error responses

#### CLI Tool (`bms-cli`)
- ✅ Commands: `init`, `store`, `recall`, `list`, `verify`, `stats`
- ✅ Configurable database path
- ✅ User-friendly output

### Build & Test

```bash
# Build all crates (release mode)
cargo build --release

# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p bms-core
cargo test -p bms-storage

# Build individual binaries
cargo build --release --bin bms      # CLI
cargo build --release --bin bms-api  # API Server
```

### Quick Development Workflow

```bash
# 1. Make changes to code
vim crates/bms-core/src/coordinate.rs

# 2. Run tests
cargo test -p bms-core

# 3. Check compilation
cargo check --workspace

# 4. Build and test CLI
cargo run --bin bms -- init
cargo run --bin bms -- store --state '{"test": 1}'

# 5. Build API and test
cargo run --bin bms-api  # starts on :3000
curl http://localhost:3000/health
```

### Adding New Features

#### Adding a New Core Function

1. Define in `crates/bms-core/src/<module>.rs`
2. Add tests in the same file (`#[cfg(test)] mod tests`)
3. Export from `lib.rs` if public API
4. Run `cargo test -p bms-core`

#### Adding a New API Endpoint

1. Add handler in `crates/bms-api/src/handlers.rs`
2. Define request/response types with `serde`
3. Register route in `crates/bms-api/src/main.rs`
4. Test with `curl` or create integration test

#### Adding a New CLI Command

1. Add variant to `Commands` enum in `crates/bms-cli/src/main.rs`
2. Implement handler in `match cli.command`
3. Test: `cargo run --bin bms -- <new-command>`

### Testing Guidelines

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests (Future)

Create `tests/integration_test.rs` in crate root:

```rust
use bms_core::*;

#[test]
fn test_end_to_end() {
    // Test full workflow
}
```

### Performance Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
cargo flamegraph --bin bms-api

# Memory profiling
cargo build --release
valgrind --tool=massif target/release/bms-api

# Benchmarking (when benches are added)
cargo bench -p bms-core
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin bms-api

# Trace-level logging for specific module
RUST_LOG=bms_core::delta=trace cargo test

# Run single test with output
cargo test test_name -- --nocapture

# Debug with lldb/gdb
rust-lldb target/debug/bms-api
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --workspace -- -D warnings

# Check for unused dependencies
cargo install cargo-udeps
cargo +nightly udeps

# Audit dependencies for security
cargo install cargo-audit
cargo audit
```

### Database Management

```bash
# Inspect SQLite database
sqlite3 bms.db
  .tables
  .schema coordinates
  SELECT * FROM coordinates LIMIT 10;
  
# Backup database
cp bms.db bms_backup_$(date +%Y%m%d).db

# Reset database
rm bms.db
cargo run --bin bms -- init
```

### API Testing

```bash
# Store a state
curl -X POST http://localhost:3000/store \
  -H "Content-Type: application/json" \
  -d '{"state": {"msg": "test"}, "author": "dev"}'

# Recall state
curl http://localhost:3000/recall/<COORD_ID>

# Verify chain
curl http://localhost:3000/verify/<COORD_ID>

# Health check
curl http://localhost:3000/health

# Get stats
curl http://localhost:3000/stats
```

### Common Issues & Solutions

#### SQLite Locked
```bash
# Multiple processes accessing same DB
# Solution: Use different DB paths or wait
```

#### Compilation Errors with sqlx
```bash
# Macro errors
# Solution: Clean and rebuild
cargo clean
cargo build
```

#### Port Already in Use
```bash
# API won't start on :3000
# Solution: Change port or kill process
lsof -ti:3000 | xargs kill
```

### Next Steps (Phase 2)

- [ ] Vector search integration (Qdrant)
- [ ] Embedding generation (sentence-transformers)
- [ ] Search endpoint with semantic similarity
- [ ] Performance benchmarks on Raspberry Pi 5
- [ ] Docker containerization
- [ ] CI/CD pipeline

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [sqlx Docs](https://docs.rs/sqlx/)
- [Axum Docs](https://docs.rs/axum/)
- [RFC 6902 (JSON Patch)](https://tools.ietf.org/html/rfc6902)

---

## Credits

**Original Architecture & Design**: TechnoShaman (Discord ID: `191470268999401472`)  
**Implementation**: Whisper Engine AI  
**Repository**: [whisperengine-ai/BMS](https://github.com/whisperengine-ai/BMS)
