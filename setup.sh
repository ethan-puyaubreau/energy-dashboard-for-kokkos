#!/bin/bash

set -e

[ -f "init-db/init_tmp.sql" ] && rm -f init-db/init_tmp.sql

if docker compose ps -q &>/dev/null; then
  docker compose down -v
fi

mkdir -p data/variorum

python3 -m venv venv
source venv/bin/activate
pip install -q numpy pandas
python3 scripts/variorum/aggregate_variorum.py
deactivate
rm -rf venv

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
sed -i '/COPY.*FROM/d' init-db/init_tmp.sql
printf "%b" "$IMPORT_SQL" >> init-db/init_tmp.sql

if [ -f "$VARIORUM_DIR/variorum_series.sql" ]; then
  cat "$VARIORUM_DIR/variorum_series.sql" >> init-db/init_tmp.sql
else
  echo "WARNING: variorum_series.sql not found."
fi

docker compose up -d

echo ""
echo "Waiting for services..."
sleep 15

echo ""
echo "--------------------------------------------------------"
echo "Grafana : http://localhost:3000  (admin / admin)"
echo "PostgreSQL : localhost:5432  energy_analysis / grafana_user"
echo "--------------------------------------------------------"
