# kokkos-energy

High-performance energy analysis and profiling tool for Kokkos applications.

`kokkos-energy` provides attribution of electrical energy (Joules) and average power (Watts) to Kokkos execution blocks (`UserRegion`, `parallel_for`, `parallel_reduce`, `parallel_scan`, and memory movements).

> **Note on Kokkos Trademark & Affiliation:**
> `kokkos-energy` is an independent, community-driven analysis tool. It is not an official project of the Kokkos ecosystem nor endorsed by the Linux Foundation.

## Key Features

- **Zero-Daemon, Zero-Docker:** Single native binary suitable for unprivileged HPC environments.
- **Hierarchical Energy Attribution:** Accurately attributes energy across enclosing regions and kernels using trapezoidal numerical integration.
- **HPC Console Report:** Formatted summary table printed directly to stdout (ideal for Slurm batch job logs).
- **Perfetto / Chrome Tracing Export:** Generate interactive `trace.json` timelines viewable at [ui.perfetto.dev](https://ui.perfetto.dev).
- **Standalone HTML Dashboard:** Export self-contained offline reports (`report.html`) with embedded interactive Plotly charts.
- **Direct Runner Mode:** Transparently launch an application and profile it in a single command.

---

## Quick Start

### 1. Build from source

```bash
cargo build --release
```

The resulting standalone binary is located at `target/release/kokkos-energy`.

### 2. Generate a trace with the profiling connector

The KokkosP connector library is currently available in the [`feat/v1-energy-profiler`](https://github.com/ethan-puyaubreau/kokkos-tools/tree/feat/v1-energy-profiler) branch of the `kokkos-tools` fork.

Compile the connector:
```bash
g++ -std=c++20 -O3 -fPIC -shared kp_energy_profiler.cpp \
    -I/usr/local/cuda/include -L/usr/lib/x86_64-linux-gnu -lnvidia-ml -lpthread \
    -o libkokkos_energy.so
```

### 3. Usage Modes

#### Mode A: Direct Runner (Recommended)

Run and analyze your Kokkos application in a single step:

```bash
kokkos-energy run --lib /path/to/libkokkos_energy.so --report report.html --perfetto trace.json -- ./my_app [args...]
```

#### Mode B: Post-Mortem Analysis

If the application was run independently:

```bash
# 1. Run with standard KokkosP environment variables
export KOKKOS_TOOLS_LIBS=/path/to/libkokkos_energy.so
export KOKKOS_TOOLS_OUTPUT_PATH=./my_trace
./my_app

# 2. Analyze the resulting directory
kokkos-energy analyze ./my_trace --report report.html --perfetto trace.json
```

Output example:

```text
  Kokkos Energy Analysis - App: energy_bench (Host: wsl-rtx3080ti, Backend: CUDA)
┌─────────────────────────────────────────────┬─────────────────┬───────┬──────────────┬────────────┬───────────────┬──────────┐
│ Block / Kernel                              ┆ Category        ┆ Calls ┆ Duration (s) ┆ Energy (J) ┆ Avg Power (W) ┆ % Energy │
╞═════════════════════════════════════════════╪═════════════════╪═══════╪══════════════╪════════════╪═══════════════╪══════════╡
│ Reduction                                   ┆ USER_REGION     ┆ 1     ┆ 4.000        ┆ 1082.61    ┆ 270.6         ┆ 50.2%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ reduce                                      ┆ PARALLEL_REDUCE ┆ 8660  ┆ 3.930        ┆ 1063.62    ┆ 270.6         ┆ 49.3%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ MatVec                                      ┆ USER_REGION     ┆ 1     ┆ 4.000        ┆ 839.08     ┆ 209.8         ┆ 38.9%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ matvec                                      ┆ PARALLEL_FOR    ┆ 8217  ┆ 3.941        ┆ 826.73     ┆ 209.8         ┆ 38.3%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ daxpy                                       ┆ PARALLEL_FOR    ┆ 39882 ┆ 1.835        ┆ 215.53     ┆ 117.5         ┆ 10.0%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Total Trace (Active)                        ┆ -               ┆ -     ┆ 10.012       ┆ 2157.92    ┆ 215.5         ┆ 100.0%   │
└─────────────────────────────────────────────┴─────────────────┴───────┴──────────────┴────────────┴───────────────┴──────────┘
```

---

## Visualizing Traces

### 1. Interactive Perfetto Timeline
Pass `--perfetto trace.json` and open [ui.perfetto.dev](https://ui.perfetto.dev). Drag-and-drop the JSON file to navigate slices with `W`, `A`, `S`, `D`.

### 2. Standalone HTML Report
Pass `--report report.html` and open the generated file directly in any web browser.

---

## Data Specification

See [DATA_SPEC.md](DATA_SPEC.md) for complete details on the underlying `events.csv`, `power_samples.csv`, and `metadata.json` schemas.
