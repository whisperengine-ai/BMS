# Babel Memory System (BMS) - Rust Implementation

A high-performance, deterministic memory system implementing telic compression for AI applications.

## 🎯 Project Overview

BMS stores the **telic act of choosing** which state matters, using:
- **Coordinates**: 128-bit deterministic addresses (base32)
- **Deltas**: RFC 6902 JSON Patch for lossless compression
- **Merkle Chains**: Tamper-evident delta linking with SHA3-256
- **Snapshots**: Periodic full-state checkpoints for fast reconstruction

**Target Performance** (Raspberry Pi 5, 8GB):
- Store: 250-400ms P50
- Recall: 300-450ms P50
- Compression: 85-97%
- Integrity: ≥99.9% deterministic reconstruction

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                       BMS System                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  bms-core   │  │ bms-storage  │  │  bms-vector  │  │
│  │             │  │              │  │              │  │
│  │ • Canonical │  │ • SQLite KV  │  │ • Embeddings │  │
│  │ • Coords    │  │ • Schema     │  │ • Search     │  │
│  │ • Deltas    │  │ • Repository │  │ • (Future)   │  │
│  │ • Merkle    │  │              │  │              │  │
│  │ • Snapshots │  │              │  │              │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
│                                                         │
│  ┌─────────────┐                    ┌──────────────┐  │
│  │  bms-api    │                    │   bms-cli    │  │
│  │             │                    │              │  │
│  │ • REST API  │                    │ • Commands   │  │
│  │ • Axum      │                    │ • Testing    │  │
│  └─────────────┘                    └──────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 📦 Workspace Structure

- **bms-core**: Core primitives (canonical JSON, coordinates, deltas, Merkle chains, snapshots)
- **bms-storage**: SQLite persistence layer
- **bms-vector**: Vector search integration (placeholder for Qdrant)
- **bms-api**: REST API server (Axum)
- **bms-cli**: Command-line interface

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- SQLite 3.x

### Build

```bash
cargo build --release
```

### Initialize Database

```bash
cargo run --bin bms -- init
```

### Store a State

```bash
cargo run --bin bms -- store --state '{"message": "Hello BMS", "value": 42}'
```

### Recall a State

```bash
# Get coordinate ID from store output
cargo run --bin bms -- recall <COORD_ID>
```

### List Coordinates

```bash
cargo run --bin bms -- list
```

### Verify Chain Integrity

```bash
cargo run --bin bms -- verify <COORD_ID>
```

### Run API Server

```bash
cargo run --bin bms-api
# Server starts on http://localhost:3000
```

## 🔌 API Endpoints

### Health Check
```bash
curl http://localhost:3000/health
```

### Store State
```bash
curl -X POST http://localhost:3000/store \
  -H "Content-Type: application/json" \
  -d '{
    "state": {"message": "Hello", "value": 42},
    "author": "system"
  }'
```

### Recall State
```bash
curl http://localhost:3000/recall/<COORD_ID>
```

### Verify Chain
```bash
curl http://localhost:3000/verify/<COORD_ID>
```

### Create Snapshot
```bash
curl -X POST http://localhost:3000/snapshot/<COORD_ID>
```

### List Coordinates
```bash
curl http://localhost:3000/coords
```

### Get Statistics
```bash
curl http://localhost:3000/stats
```

## 🧪 Testing

Run all tests:
```bash
cargo test --workspace
```

Run tests for specific crate:
```bash
cargo test -p bms-core
```

Run with logging:
```bash
RUST_LOG=debug cargo test
```

## 📊 Benchmarking

```bash
cargo bench -p bms-core
```

## 🔧 Configuration

### Environment Variables

- `BMS_DB_PATH`: Database file path (default: `./bms.db`)
- `RUST_LOG`: Logging level (default: `info`)

### Database Path

Via CLI:
```bash
cargo run --bin bms -- --db-path /path/to/bms.db list
```

Via environment:
```bash
export BMS_DB_PATH=/path/to/bms.db
cargo run --bin bms-api
```

## 📈 POC/MVP Scope

### Phase 1: Core Engine ✅
- [x] Canonical JSON serialization
- [x] Coordinate generation (128-bit, base32)
- [x] Delta compression (RFC 6902)
- [x] Merkle chain verification
- [x] Snapshot management
- [x] SQLite storage

### Phase 2: API & Search (In Progress)
- [x] REST API with Axum
- [x] Basic endpoints (store, recall, verify)
- [ ] Vector search integration
- [ ] Embedding generation

### Phase 3: Benchmarking (Planned)
- [ ] Performance testing suite
- [ ] Compression ratio analysis
- [ ] Raspberry Pi 5 validation
- [ ] Benchmark report

### Deferred Post-MVP
- OCR pipeline (DeepSeek)
- HRM/Fabric normalization
- Resonance layer (RCS-M)
- LLM integration
- Security (signing, encryption)
- PII redaction
- Multi-tenancy
- Schema migration

## 🔬 Design Principles

1. **Determinism**: Same input → same coordinate (canonical JSON, fixed hashing)
2. **Lossless**: Perfect state reconstruction via deltas
3. **Verifiable**: Merkle chains ensure tamper-evidence
4. **Efficient**: 85-97% compression through delta storage
5. **Edge-Ready**: Optimized for resource-constrained hardware

## 📚 Technical Details

### Coordinate Generation
```
seed = SHA3-256(canonical_state + "|" + ISO8601_UTC)[:16]
coord_id = base32(seed)  // 26 characters
```

### Delta Compression
```
delta = json_patch::diff(prev_state, current_state)
delta_hash = SHA3-256(canonical(delta))
```

### Merkle Chain
```
chain_hash = SHA3-256(parent_hash + current_delta_hash)
```

### Reconstruction
```
state = snapshot.state
for delta in forward_deltas:
    apply(delta.ops, state)
```

## 🐛 Troubleshooting

### Build Errors

If you encounter SQLite linking issues:
```bash
# macOS
brew install sqlite3

# Ubuntu/Debian
sudo apt-get install libsqlite3-dev
```

### Database Locked

Multiple processes accessing the same database:
```bash
# Use different database paths
BMS_DB_PATH=./bms-test.db cargo run --bin bms-cli
```

## 🤝 Contributing

This is a POC/MVP implementation. Focus areas:
- Performance optimization
- Test coverage
- Documentation
- Vector search integration

## 📄 License

MIT License - See LICENSE file

## 🔗 References

- RFC 6902: JSON Patch
- RFC 4648: Base32 Encoding
- SHA3-256: NIST FIPS 202
- CTMU: Cognitive-Theoretic Model of the Universe
- SLMU: Soteriological Logical Meta-Unification

---

**Status**: POC/MVP Phase 1 Complete ✅

Built with ❤️ in Rust for edge AI applications.

---

## Credits

**Original Architecture & Design**: TechnoShaman (Discord ID: `191470268999401472`)  
**Implementation**: Whisper Engine AI  
**Repository**: [whisperengine-ai/BMS](https://github.com/whisperengine-ai/BMS)
