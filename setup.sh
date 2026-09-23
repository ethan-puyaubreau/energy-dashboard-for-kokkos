#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -f ".env" ]; then
  # shellcheck disable=SC2046
  export $(grep -v '^#' .env | xargs)
elif [ -f ".env.example" ]; then
  echo "INFO: No .env found, using defaults from .env.example"
  # shellcheck disable=SC2046
  export $(grep -v '^#' .env.example | xargs)
fi

[ -f "init-db/init_tmp.sql" ] && rm -f init-db/init_tmp.sql

if docker compose ps -q &>/dev/null; then
  docker compose down -v
fi

mkdir -p data/variorum

VENV_DIR=".venv"
if [ ! -d "$VENV_DIR" ]; then
  echo "Creating Python virtual environment in $VENV_DIR..."
  python3 -m venv "$VENV_DIR"
  # shellcheck source=/dev/null
  source "$VENV_DIR/bin/activate"
  pip install --upgrade pip -q
  pip install -q -r requirements.txt
else
  # shellcheck source=/dev/null
  source "$VENV_DIR/bin/activate"
fi

python3 scripts/variorum/aggregate_variorum.py
deactivate

VARIORUM_DIR="data/variorum"
VARIORUM_FILES=("variorum_relative.csv" "variorum_absolute.csv" "variorum_gpus.csv" "variorum_kernels.csv" "variorum_stats.csv" "variorum_regions.csv")

ANY_VARIORUM_FOUND=false
for f in "${VARIORUM_FILES[@]}"; do
  [ -f "$VARIORUM_DIR/$f" ] && ANY_VARIORUM_FOUND=true
done
[ "$ANY_VARIORUM_FOUND" = false ] && echo "WARNING: No Variorum data found in $VARIORUM_DIR." >&2

IMPORT_SQL=""
for f in "${VARIORUM_FILES[@]}"; do
  if [ -f "$VARIORUM_DIR/$f" ]; then
    TABLE_NAME="${f%.csv}"
    IMPORT_SQL+="\\COPY $TABLE_NAME FROM '/csv_data/variorum/$f' WITH (FORMAT csv, HEADER true);\n"
  else
    echo "WARNING: $f not found, skipping import."
  fi
done

cat init-db/init.sql > init-db/init_tmp.sql
printf "%b" "$IMPORT_SQL" >> init-db/init_tmp.sql

if [ -f "$VARIORUM_DIR/variorum_series.sql" ]; then
  cat "$VARIORUM_DIR/variorum_series.sql" >> init-db/init_tmp.sql
else
  echo "WARNING: variorum_series.sql not found."
fi

docker compose up -d

echo ""
echo "Waiting for PostgreSQL to be healthy..."
RETRY_COUNT=0
MAX_RETRIES=30
until docker compose exec -T postgres_db pg_isready -U "${POSTGRES_USER:-grafana_user}" -d "${POSTGRES_DB:-energy_analysis}" &>/dev/null; do
  RETRY_COUNT=$((RETRY_COUNT + 1))
  if [ "$RETRY_COUNT" -ge "$MAX_RETRIES" ]; then
    echo "ERROR: PostgreSQL service failed to become healthy within timeout." >&2
    exit 1
  fi
  sleep 1
done

echo "Services are ready."
echo ""
echo "--------------------------------------------------------"
echo "Grafana : http://localhost:3000  (admin / ${GRAFANA_ADMIN_PASSWORD:-admin})"
echo "PostgreSQL : localhost:5432  ${POSTGRES_DB:-energy_analysis} / ${POSTGRES_USER:-grafana_user}"
echo "--------------------------------------------------------"
