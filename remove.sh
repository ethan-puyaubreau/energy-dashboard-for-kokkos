#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

[ -f "init-db/init_tmp.sql" ] && rm -f init-db/init_tmp.sql

if docker compose ps -q &>/dev/null; then
    echo "Stopping containers..."
    docker compose down -v
fi

[ -d "data" ] && rm -rf data/

find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true

echo "Done. Run ./setup.sh to restart."
