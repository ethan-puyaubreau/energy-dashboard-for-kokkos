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
- **Standalone HTML Dashboard:** Export self-contained offline reports (`report.html`) with embedded interactive Plotly charts, usable on compute nodes without network access.
- **Direct Runner Mode:** Transparently launch an application and profile it in a single command.

---

## Requirements

Energy is measured on **NVIDIA GPUs only**. CPU packages, DRAM and other
accelerators are not sampled by the connector.

### Analysis tool (`kokkos-energy`)

- Prebuilt release binary: Linux x86_64, statically linked, no runtime dependency.
- Build from source: Rust 1.88 or newer. The `analyze` command runs on any platform
  supported by Rust, the `run` command needs a platform where the connector runs.

### Profiling connector (`libkokkos_energy.so`)

- Linux x86_64, native or WSL2.
- NVIDIA driver providing NVML (`libnvidia-ml.so`, installed with the driver). Power
  readings do not require root privileges.
- CUDA toolkit headers for `nvml.h`, by default in `/usr/local/cuda/include`.
- GCC 10 or newer, or any compiler supporting `-std=c++20`.
- Kokkos built with Kokkos Tools support, which is enabled by default.

### Benchmark application (optional)

- CMake 3.16 or newer.
- Kokkos installed with the CUDA backend (`Kokkos_ENABLE_CUDA=ON`) and the
  `Kokkos_ARCH_*` option matching the GPU.
- On WSL2, the WSL-Ubuntu CUDA toolkit, not the generic Linux one which ships its own
  driver.

### Tested configuration

| Component | Version |
| :--- | :--- |
| GPU | NVIDIA GeForce RTX 3080 Ti (Ampere) |
| System | Ubuntu on WSL2 |
| Kokkos | 5.2.2, CUDA backend |

---

## Quick Start

### 1. Install

Download the static Linux x86_64 binary from the
[releases page](https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/releases),
check its checksum and extract it:

```bash
sha256sum -c kokkos-energy-v0.2.0-x86_64-unknown-linux-musl.tar.gz.sha256
tar xzf kokkos-energy-v0.2.0-x86_64-unknown-linux-musl.tar.gz
```

Or build from source:

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

The raw trace is written to a temporary directory and removed on exit. Pass
`--keep-trace <DIR>` to keep the CSV files for a later `analyze`. The exit code
of the application is forwarded, so batch jobs still see its failures.

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

Output example (rows below 0.1% trimmed):

```text
  Kokkos Energy Analysis - App: energy_bench (Host: wsl-rtx3080ti, Backend: CUDA)
┌─────────────────────────────────────────────┬─────────────────┬───────┬──────────────┬─────────────────┬─────────────────┬───────────────┬────────┐
│ Block / Kernel                              ┆ Category        ┆ Calls ┆ Duration (s) ┆ Energy Incl (J) ┆ Energy Self (J) ┆ Avg Power (W) ┆ % Self │
╞═════════════════════════════════════════════╪═════════════════╪═══════╪══════════════╪═════════════════╪═════════════════╪═══════════════╪════════╡
│ Reduction                                   ┆ USER_REGION     ┆ 1     ┆ 4.000        ┆ 1082.61         ┆ 18.99           ┆ 270.6         ┆ 0.9%   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ reduce                                      ┆ PARALLEL_REDUCE ┆ 8660  ┆ 3.930        ┆ 1063.62         ┆ 1063.62         ┆ 270.6         ┆ 49.3%  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ MatVec                                      ┆ USER_REGION     ┆ 1     ┆ 4.000        ┆ 839.08          ┆ 12.37           ┆ 209.8         ┆ 0.6%   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ matvec                                      ┆ PARALLEL_FOR    ┆ 8217  ┆ 3.941        ┆ 826.72          ┆ 826.72          ┆ 209.8         ┆ 38.3%  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ daxpy                                       ┆ PARALLEL_FOR    ┆ 39882 ┆ 1.835        ┆ 215.53          ┆ 215.53          ┆ 117.5         ┆ 10.0%  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ Idle (outside events)                       ┆ -               ┆ -     ┆ 0.177        ┆ 20.83           ┆ 20.83           ┆ 117.9         ┆ 1.0%   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ Total Trace                                 ┆ -               ┆ -     ┆ 10.014       ┆ 2158.32         ┆ 2158.32         ┆ 215.5         ┆ 100.0% │
└─────────────────────────────────────────────┴─────────────────┴───────┴──────────────┴─────────────────┴─────────────────┴───────────────┴────────┘
  GPU 0: 2158.32 J, 215.5 W avg

  Note: 100.0% of events are shorter than the 20.4 ms sampling period, their power is interpolated between samples rather than measured.
```

---

## Reading the Report

- **Energy Incl** is the energy spent while a block was active, children included.
- **Energy Self** is the energy spent in the block itself, children excluded. The
  `% Self` column is based on it, so percentages sum to 100% with the idle row.
- **Idle (outside events)** is the energy measured while no instrumented block was
  running, including before the first and after the last event.
- **Avg Power** is the inclusive energy divided by the block duration.
- The lines under the table give the energy of each measured device. The table
  sums all of them.
- A final note appears when events are shorter than the sampling period, with the
  share of such events. See Limits below.

### Attribution Rules

- Each power series (one per domain and device) is integrated on its own with the
  trapezoidal rule, then the series are summed.
- Power is linearly interpolated at block boundaries.
- Sampled power is the only reference. Hardware cumulative energy counters are
  ignored because they proved unreliable on NVIDIA GPUs.
- Blocks that overlap without being nested, such as concurrent kernels, share the
  energy of the overlapping interval equally.

### Multi-Rank Traces

Under MPI or Slurm the connector writes one `rank_<N>` subdirectory per rank.
`analyze` and `run` detect them and print one report per rank, in rank order.
Exports are written once per rank with the rank appended to the file name, for
instance `--report report.html` produces `report_rank_0.html`, `report_rank_1.html`.

Each rank samples every GPU visible on its node. Ranks sharing a node therefore
report the same device energy: do not sum device or total energies across ranks.

### Limits

- The connector samples power every 20 ms. Most kernels are shorter than that, so
  their power is interpolated between two samples rather than measured. Per-kernel
  figures are reliable in aggregate over many calls, not for a single launch. The
  report prints the share of affected events so this is never silent.
- Overlapping blocks share energy equally because a device reports a single power
  value. With the default Kokkos global fencing, kernels do not overlap and this
  rule never applies.

---

## Visualizing Traces

### 1. Interactive Perfetto Timeline
Pass `--perfetto trace.json` and open [ui.perfetto.dev](https://ui.perfetto.dev). Drag-and-drop the JSON file to navigate slices with `W`, `A`, `S`, `D`. Nested blocks are stacked on a single track. Blocks that overlap without being nested are moved to an extra track, and each device gets its own power counter.

### 2. Standalone HTML Report
Pass `--report report.html` and open the generated file directly in any web browser.

---

## Data Specification

See [DATA_SPEC.md](DATA_SPEC.md) for complete details on the underlying `events.csv`, `power_samples.csv`, and `metadata.json` schemas.
