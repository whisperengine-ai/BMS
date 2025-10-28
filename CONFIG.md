# BMS Configuration

## Core Settings

```yaml
bms:
  coord:
    bytes: 16              # 128-bit coordinate
    encoding: base32       # RFC 4648, no padding
  
  compression:
    level: L2              # L0=raw, L1=markup, L2=semantic delta (MVP)
    delta_window: 10
    snapshot_interval: 128
    large_delta_bytes: 4096
  
  integrity:
    hash: sha3_256
    merkle: true
    sign: false            # Post-MVP: ed25519
  
  storage:
    kv: sqlite             # Embedded SQLite
    db_path: ./bms.db
    objects_path: ./objects
  
  search:
    enabled: false         # Post-MVP
    provider: qdrant
    hnsw:
      M: 32
      ef_construction: 200
      ef_search: 128
```

## Runtime Configuration

```yaml
runtime:
  hardware: generic
  log_level: info
  max_connections: 5
```

## Environment Variables

```bash
# Database
BMS_DB_PATH=./bms.db

# Logging
RUST_LOG=info

# API Server
BMS_API_HOST=0.0.0.0
BMS_API_PORT=3000
```

## Development

```yaml
dev:
  snapshot_interval: 10   # Faster snapshots for testing
  verbose_logging: true
```

## Production (Future)

```yaml
prod:
  snapshot_interval: 128
  encryption_at_rest: true
  sign_deltas: true
  multi_tenancy: true
```

---

## Credits

**Original Architecture & Design**: TechnoShaman (Discord ID: `191470268999401472`)  
**Implementation**: Whisper Engine AI  
**Repository**: [whisperengine-ai/BMS](https://github.com/whisperengine-ai/BMS)
