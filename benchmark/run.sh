#!/bin/bash
# Run energy_bench with Variorum profiling and stage output for setup.sh.
#
# Required env vars:
#   VARIORUM_ENERGY_LIB  — path to libVariorumEnergyProfiler.so
#
# Optional:
#   VARIORUM_LIB_DIR     — Variorum lib dir (default: dirname of VARIORUM_ENERGY_LIB)
#   BATCH_NAME           — output subfolder name under input/variorum/ (default: energy_bench_1)
#
# After this script, run ./setup.sh from the repo root to load the data into Grafana.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY="$SCRIPT_DIR/build/energy_bench"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: $BINARY not found — run ./build.sh first." >&2
    exit 1
fi

if [ -z "$VARIORUM_ENERGY_LIB" ]; then
    echo "ERROR: VARIORUM_ENERGY_LIB is not set." >&2
    echo "  export VARIORUM_ENERGY_LIB=/path/to/libVariorumEnergyProfiler.so" >&2
    exit 1
fi

VARIORUM_LIB_DIR="${VARIORUM_LIB_DIR:-$(dirname "$VARIORUM_ENERGY_LIB")}"
BATCH_NAME="${BATCH_NAME:-energy_bench_1}"
OUTPUT_DIR="$REPO_ROOT/input/variorum/$BATCH_NAME"

# On WSL2 the NVML stub lives outside the standard linker paths.
# Variorum won't find libnvidia-ml.so without it.
WSL_LIB=""
if grep -qi microsoft /proc/version 2>/dev/null; then
    WSL_LIB="/usr/lib/wsl/lib"
    if [ ! -d "$WSL_LIB" ]; then
        echo "WARNING: WSL2 detected but $WSL_LIB not found." >&2
        echo "  Install the CUDA-WSL toolkit: https://developer.nvidia.com/cuda-downloads (select WSL-Ubuntu)" >&2
    else
        echo "[wsl2] adding $WSL_LIB to LD_LIBRARY_PATH for NVML"
        echo "[wsl2] note: RAPL/CPU energy will be unavailable (no /dev/msr in WSL2)"
    fi
fi

mkdir -p "$OUTPUT_DIR"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$TMPDIR"
LD_LIBRARY_PATH="${WSL_LIB:+$WSL_LIB:}$VARIORUM_LIB_DIR:$LD_LIBRARY_PATH" \
    KOKKOS_TOOLS_LIBS="$VARIORUM_ENERGY_LIB" \
    "$BINARY"

HOSTNAME=$(hostname)
shopt -s nullglob
FILES=("${HOSTNAME}"-variorum-*)
if [ ${#FILES[@]} -eq 0 ]; then
    echo "WARNING: No Variorum output files found (expected ${HOSTNAME}-variorum-*)." >&2
    echo "  Check that VARIORUM_ENERGY_LIB points to the Kokkos energy profiler plugin." >&2
else
    mv "${FILES[@]}" "$OUTPUT_DIR/"
    echo "Moved ${#FILES[@]} file(s) to $OUTPUT_DIR/"
fi
shopt -u nullglob

echo "Done. Run '$REPO_ROOT/setup.sh' to load into Grafana."
