#!/bin/bash

set -e

[ -f "init-db/init_tmp.sql" ] && rm -f init-db/init_tmp.sql

if docker compose ps -q &>/dev/null; then
    echo "Stopping containers..."
    docker compose down
fi

if docker volume ls -q | grep -q "grafana_energy"; then
    docker compose down -v
fi

[ -d "data" ] && rm -rf data/

find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
docker system prune -f &>/dev/null

echo "Done. Run ./setup.sh to restart."
