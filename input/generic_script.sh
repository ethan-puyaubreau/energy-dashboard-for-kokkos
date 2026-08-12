#!/bin/bash

# --- Configuration ---
NUM_ITERATIONS=1
COMMAND="YOUR KOKKOS COMMAND HERE"

# --- Variorum Library ---
VARIORUM_ENERGY_LIB="YOUR VARIORUM ENERGY PROFILER LIBRARY PATH HERE"
VARIORUM_DIR="YOUR VARIORUM INSTALLATION DIRECTORY HERE"

HOSTNAME=$(hostname)
OUTPUT_DIR_PREFIX="energy_bench"

run_variorum_power() {
    for i in $(seq 1 $NUM_ITERATIONS); do
        BATCH_DIR="${OUTPUT_DIR_PREFIX}_variorum_power_${i}"
        mkdir -p "$BATCH_DIR"
        echo "Running Variorum Power iteration $i... out of $NUM_ITERATIONS"
        LD_LIBRARY_PATH="$VARIORUM_DIR/lib:$LD_LIBRARY_PATH" KOKKOS_TOOLS_LIBS="$VARIORUM_ENERGY_LIB" $COMMAND
        shopt -s nullglob
        mv ${HOSTNAME}* "$BATCH_DIR/" 2>/dev/null
        mkdir -p ./variorum/
        mv ${BATCH_DIR} ./variorum/ 2>/dev/null
        shopt -u nullglob
    done
}

echo "Starting EnergyBench runs..."
run_variorum_power
echo "All EnergyBench runs completed."