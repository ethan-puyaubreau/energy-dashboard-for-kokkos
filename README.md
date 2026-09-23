# Energy Analysis Dashboard for Kokkos
Author: Ethan Puyaubreau

Grafana + PostgreSQL dashboard for Variorum energy profiling data, provisioned automatically via Docker Compose.

## Quick Start

**Prerequisites:** Docker, Docker Compose, Python 3, Variorum library.

1. Clone the repository.
2. (Optional) Configure environment variables:
   ```bash
   cp .env.example .env
   ```
3. Place CSV output files in `input/variorum/` (see naming below), or run `input/generic_script.sh` to generate them.
4. Run `./setup.sh` — creates or reuses `.venv`, aggregates the data, starts the stack, and waits for database readiness.
5. Open `http://localhost:3000` (default: admin / admin, configurable in `.env`).

## Input File Naming

Files must match these suffixes inside `input/variorum/` (subdirectories supported):

| Suffix | Content |
|--------|---------|
| `*-variorum-power-relative.csv` | Relative timestamps + power |
| `*-variorum-power.csv` | Absolute timestamps + power |
| `*-variorum-power-gpus.csv` | Per-GPU power |
| `*-variorum-power-kernels.csv` | Kernel timings |
| `*-variorum-power-regions.csv` | Region timings |
| `*-variorum-power.dat` | Summary stats |

The Kokkos Variorum tool generates these names by default.

## Stop / Cleanup

```bash
# Stop, keep data
docker compose down

# Full cleanup (containers, volumes, data/)
./remove.sh

# Clear input files only
./input/clean.sh
```
