# BMS Deployment Guide

## Prerequisites

### System Requirements

**Minimum**:
- CPU: 2 cores
- RAM: 2 GB
- Storage: 10 GB
- OS: Linux, macOS, or Windows (WSL2)

**Recommended** (Raspberry Pi 5):
- CPU: ARM Cortex-A76 (4 cores)
- RAM: 8 GB
- Storage: 32 GB (SD card or SSD)
- OS: Ubuntu 24.04 LTS (ARM64)

### Software Dependencies

**Required**:
- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- SQLite 3.x (usually pre-installed)
- Git (for source checkout)

**Optional**:
- Docker 24+ (for containerized deployment)
- systemd (for service management)

## Installation

### Option 1: Build from Source

```bash
# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/whisperengine-ai/BMS.git
cd BMS

### Option 2: Download Pre-built Binary

```bash
# Download latest release (adjust version as needed)
curl -LO https://github.com/whisperengine-ai/BMS/releases/download/v0.1.0/bms-linux-aarch64.tar.gz
tar xzf bms-linux-aarch64.tar.gz

### Option 3: Docker (Future)

```bash
# Pull image
docker pull ghcr.io/markcastillo/bms:latest

# Run API server
docker run -d -p 3000:3000 -v $(pwd)/data:/data ghcr.io/markcastillo/bms:latest
```

## Configuration

### Environment Variables

```bash
# Database location
export BMS_DB_PATH=/var/lib/bms/bms.db

# API server host/port
export BMS_API_HOST=0.0.0.0
export BMS_API_PORT=3000

# Logging
export RUST_LOG=info                    # info, debug, trace
export RUST_LOG=bms_core=debug,bms_storage=trace  # per-module
```

### Database Initialization

```bash
# Create data directory
sudo mkdir -p /var/lib/bms
sudo chown $USER:$USER /var/lib/bms

# Initialize database
BMS_DB_PATH=/var/lib/bms/bms.db bms init
```

## Running the Services

### CLI Tool

```bash
# One-time commands
bms --db-path /var/lib/bms/bms.db store --state '{"key": "value"}'
bms --db-path /var/lib/bms/bms.db list
bms --db-path /var/lib/bms/bms.db stats
```

### API Server (Development)

```bash
# Run in foreground
BMS_DB_PATH=/var/lib/bms/bms.db bms-api

# Test health endpoint
curl http://localhost:3000/health
```

### API Server (Production with systemd)

Create `/etc/systemd/system/bms-api.service`:

```ini
[Unit]
Description=BMS API Server
After=network.target

[Service]
Type=simple
User=bms
Group=bms
WorkingDirectory=/opt/bms
Environment="BMS_DB_PATH=/var/lib/bms/bms.db"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/bms-api
Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/bms

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
# Create service user
sudo useradd -r -s /bin/false bms
sudo chown -R bms:bms /var/lib/bms

# Install service
sudo systemctl daemon-reload
sudo systemctl enable bms-api
sudo systemctl start bms-api

# Check status
sudo systemctl status bms-api
sudo journalctl -u bms-api -f
```

## Raspberry Pi 5 Deployment

### 1. Install Ubuntu 24.04 LTS (ARM64)

```bash
# Flash SD card with Ubuntu Server 24.04 ARM64
# Use Raspberry Pi Imager or balenaEtcher

# Boot Pi and update
sudo apt update && sudo apt upgrade -y
```

### 2. Install Rust

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 3. Install Dependencies

```bash
# Install build essentials
sudo apt install -y build-essential pkg-config libsqlite3-dev git

# Optional: install sqlite3 CLI
sudo apt install -y sqlite3
```

### 4. Build BMS

```bash
# Clone and build
cd ~
git clone https://github.com/whisperengine-ai/BMS.git
cd BMS
cargo build --release

# This will take 5-10 minutes on Pi 5
```

### 5. Deploy

```bash
# Copy binaries
sudo cp target/release/bms /usr/local/bin/
sudo cp target/release/bms-api /usr/local/bin/

# Create data directory
sudo mkdir -p /var/lib/bms
sudo chown pi:pi /var/lib/bms

# Initialize
bms --db-path /var/lib/bms/bms.db init

# Set up systemd service (see above)
```

### 6. Optimize for Pi 5

#### Enable WAL mode for better concurrency:

```bash
sqlite3 /var/lib/bms/bms.db 'PRAGMA journal_mode=WAL;'
```

#### Tune kernel parameters in `/etc/sysctl.conf`:

```ini
# Increase file descriptors
fs.file-max = 65536

# Tune network
net.core.somaxconn = 1024
net.ipv4.tcp_max_syn_backlog = 2048
```

Apply:
```bash
sudo sysctl -p
```

## Monitoring & Maintenance

### Health Checks

```bash
# API health
curl http://localhost:3000/health

# Database integrity
sqlite3 /var/lib/bms/bms.db 'PRAGMA integrity_check;'

# Check stats
curl http://localhost:3000/stats
```

### Logs

```bash
# systemd logs
sudo journalctl -u bms-api -f

# With filtering
sudo journalctl -u bms-api --since "1 hour ago" | grep ERROR
```

### Database Backup

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR=/var/backups/bms
mkdir -p $BACKUP_DIR
DATE=$(date +%Y%m%d_%H%M%S)

# Stop API (optional for consistency)
sudo systemctl stop bms-api

# Backup database
sqlite3 /var/lib/bms/bms.db ".backup $BACKUP_DIR/bms_$DATE.db"

# Compress
gzip $BACKUP_DIR/bms_$DATE.db

# Start API
sudo systemctl start bms-api

# Keep only last 30 days
find $BACKUP_DIR -name "bms_*.db.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/bms_$DATE.db.gz"
```

Add to crontab:
```bash
# Daily backup at 2 AM
0 2 * * * /opt/bms/backup.sh
```

### Database Maintenance

```bash
# Vacuum database (reclaim space)
sqlite3 /var/lib/bms/bms.db 'VACUUM;'

# Analyze for query optimization
sqlite3 /var/lib/bms/bms.db 'ANALYZE;'

# Check database size
ls -lh /var/lib/bms/bms.db
```

## Performance Tuning

### SQLite Optimizations

```sql
-- In database
PRAGMA journal_mode = WAL;         -- Write-ahead logging
PRAGMA synchronous = NORMAL;       -- Balance safety/performance
PRAGMA temp_store = MEMORY;        -- Use RAM for temp tables
PRAGMA mmap_size = 268435456;      -- 256MB memory map
PRAGMA cache_size = -64000;        -- 64MB page cache
```

### Rust/Tokio Tuning

Environment variables:
```bash
# Worker threads (default: CPU cores)
export TOKIO_WORKER_THREADS=4

# Stack size per task
export RUST_MIN_STACK=2097152  # 2MB
```

### API Server Tuning

Modify in `crates/bms-api/src/main.rs`:
```rust
// Increase connection pool
SqlitePoolOptions::new()
    .max_connections(20)  // up from 5
    .connect_with(options)
    .await?
```

## Scaling Considerations

### Single Instance (Current POC)

- **Capacity**: ~10K requests/hour
- **Storage**: ~100 GB for 1M coordinates
- **Limitation**: SQLite single-writer

### Future Multi-Instance

For production scaling:
1. **Migrate to PostgreSQL** for multi-writer support
2. **Add Redis** for caching hot coordinates
3. **Deploy multiple API instances** behind load balancer
4. **Separate vector search** to dedicated Qdrant cluster

## Security Hardening

### Firewall

```bash
# Allow only local access
sudo ufw allow from 127.0.0.1 to any port 3000

# Or specific subnet
sudo ufw allow from 192.168.1.0/24 to any port 3000

# Enable firewall
sudo ufw enable
```

### Reverse Proxy (nginx)

```nginx
server {
    listen 80;
    server_name bms.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # Rate limiting
        limit_req zone=api_limit burst=20 nodelay;
    }
}

# Define rate limit zone in http block
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
```

### File Permissions

```bash
# Restrict database access
chmod 600 /var/lib/bms/bms.db
chown bms:bms /var/lib/bms/bms.db

# Restrict binary execution
chmod 755 /usr/local/bin/bms*
chown root:root /usr/local/bin/bms*
```

## Troubleshooting

### API Won't Start

```bash
# Check if port is in use
lsof -i :3000

# Check database permissions
ls -l /var/lib/bms/bms.db

# Check logs
sudo journalctl -u bms-api --no-pager
```

### Database Locked

```bash
# Check for stale locks
fuser /var/lib/bms/bms.db

# Force unlock (if safe)
rm /var/lib/bms/bms.db-shm
rm /var/lib/bms/bms.db-wal
```

### High Memory Usage

```bash
# Check process memory
ps aux | grep bms-api

# Reduce connection pool size
# Edit source: SqlitePoolOptions::new().max_connections(5)

# Restart service
sudo systemctl restart bms-api
```

### Slow Queries

```bash
# Enable query logging
export RUST_LOG=sqlx::query=debug

# Analyze slow queries in logs
sudo journalctl -u bms-api | grep "slow_query"

# Add missing indexes (check schema.rs)
```

## Upgrading

### Version Upgrade

```bash
# 1. Backup database
./backup.sh

# 2. Stop service
sudo systemctl stop bms-api

# 3. Build new version
cd ~/BMS
git pull
cargo build --release

# 4. Replace binaries
sudo cp target/release/bms* /usr/local/bin/

# 5. Run migrations (if any)
bms migrate

# 6. Start service
sudo systemctl start bms-api

# 7. Verify
curl http://localhost:3000/health
```

## Uninstallation

```bash
# Stop service
sudo systemctl stop bms-api
sudo systemctl disable bms-api
sudo rm /etc/systemd/system/bms-api.service
sudo systemctl daemon-reload

# Remove binaries
sudo rm /usr/local/bin/bms*

# Remove data (WARNING: permanent)
sudo rm -rf /var/lib/bms

# Remove user
sudo userdel bms

# Remove source
rm -rf ~/BMS
```

## Support

- **Issues**: https://github.com/whisperengine-ai/BMS/issues
- **Discussions**: https://github.com/whisperengine-ai/BMS/discussions
- **Documentation**: See README.md and other docs in the repository

## Appendix: Docker Deployment (Future)

### Dockerfile

```dockerfile
FROM rust:1.75-alpine as builder
WORKDIR /build
COPY . .
RUN apk add --no-cache musl-dev sqlite-dev
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache sqlite-libs
COPY --from=builder /build/target/release/bms-api /usr/local/bin/
ENV BMS_DB_PATH=/data/bms.db
EXPOSE 3000
CMD ["bms-api"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  bms-api:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
    environment:
      - BMS_DB_PATH=/data/bms.db
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

Run:
```bash
docker-compose up -d
```

---

**Last Updated**: October 28, 2025  
**Version**: 0.1.0

---

## Credits

**Original Architecture & Design**: TechnoShaman (Discord ID: `191470268999401472`)  
**Implementation**: Whisper Engine AI  
**Repository**: [whisperengine-ai/BMS](https://github.com/whisperengine-ai/BMS)  
