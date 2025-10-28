#!/usr/bin/env bash
# BMS Quick Start Demo
set -e

echo "=== BMS (Babel Memory System) Quick Start ==="
echo

# Clean up any previous test database
rm -f demo.db

echo "1. Initializing database..."
cargo run --release --bin bms -- --db-path demo.db init
echo

echo "2. Storing first state..."
COORD1=$(cargo run --release --bin bms -- --db-path demo.db store \
  --state '{"message": "Hello BMS!", "version": 1, "author": "system"}' \
  2>&1 | grep "Coordinate:" | awk '{print $2}')
echo "   Created coordinate: $COORD1"
echo

echo "3. Storing second state (delta compression in action)..."
cargo run --release --bin bms -- --db-path demo.db store \
  --state '{"message": "Hello BMS!", "version": 2, "author": "system", "timestamp": "2025-10-28"}' \
  --coord "$COORD1"
echo

echo "4. Storing third state..."
cargo run --release --bin bms -- --db-path demo.db store \
  --state '{"message": "Hello World!", "version": 3, "author": "user", "timestamp": "2025-10-28", "status": "active"}' \
  --coord "$COORD1"
echo

echo "5. Recalling current state..."
cargo run --release --bin bms -- --db-path demo.db recall "$COORD1"
echo

echo "6. Verifying chain integrity..."
cargo run --release --bin bms -- --db-path demo.db verify "$COORD1"
echo

echo "7. Listing all coordinates..."
cargo run --release --bin bms -- --db-path demo.db list
echo

echo "8. Getting storage statistics..."
cargo run --release --bin bms -- --db-path demo.db stats
echo

echo "9. Semantic search (CLI local index)..."
echo "   Query: 'Hello World'"
cargo run --release --bin bms -- --db-path demo.db search "Hello World" --limit 5 --min-score 0.2
echo
echo "   Query: 'active'"
cargo run --release --bin bms -- --db-path demo.db search "active" --limit 5
echo

echo "=== Demo Complete! ==="
echo "Database saved at: demo.db"
echo "You can explore it further with: cargo run --bin bms -- --db-path demo.db <command>"
