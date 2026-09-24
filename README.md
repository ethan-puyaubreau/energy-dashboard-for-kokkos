# energy-dashboard-for-kokkos

[![CI](https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/actions/workflows/ci.yml/badge.svg)](https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/actions/workflows/ci.yml)

Attributes measured GPU energy to the regions and kernels of a Kokkos application.

`energy-dashboard-for-kokkos` provides attribution of electrical energy (Joules) and average power (Watts) to Kokkos execution blocks (`UserRegion`, `parallel_for`, `parallel_reduce`, `parallel_scan`, and memory movements).

> **Note on Kokkos Trademark & Affiliation:**
> `energy-dashboard-for-kokkos` is an independent analysis tool. It is not an official project of the Kokkos ecosystem nor endorsed by the Linux Foundation.

## Features

- **Single binary:** no daemon, no container, runs unprivileged on a compute node.
- **Hierarchical attribution:** energy of each region and kernel, inclusive and exclusive, by trapezoidal integration of the power trace (see Limits for what that can and cannot resolve).
- **Console report:** a summary table on stdout, readable in a Slurm job log.
- **Perfetto / Chrome Tracing Export:** Generate interactive `trace.json` timelines viewable at [ui.perfetto.dev](https://ui.perfetto.dev).
- **Standalone HTML Dashboard:** Export self-contained offline reports (`report.html`) with embedded interactive Plotly charts, usable on compute nodes without network access.
- **Direct Runner Mode:** Transparently launch an application and profile it in a single command.

---

## Requirements

Energy is measured on **NVIDIA GPUs only**. CPU packages, DRAM and other
accelerators are not sampled by the connector.

### Analysis tool (`energy-dashboard-for-kokkos`)

- Prebuilt release binary: Linux x86_64, statically linked, no runtime dependency.
- Build from source: Rust 1.88 or newer. The `analyze` command runs on any platform
  supported by Rust, the `run` command needs a platform where the connector runs.

### Profiling connector (`libenergy_dashboard_connector.so`)

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

| GPU | What was tested |
| :--- | :--- |
| NVIDIA GeForce RTX 3080 Ti (Ampere) | Connector and analysis, Ubuntu on WSL2, Kokkos 5.2.2 CUDA backend |
| NVIDIA H100 NVL (Hopper) | Analysis of the 128 ArborX DBSCAN runs traced in 2025 and [published with the SMC 2025 poster](https://github.com/ethan-puyaubreau/smc2025-gpu-energy-poster#data-and-reproduction), converted to this format; one run is a regression test |

---

## Quick start

### 1. Install

Download the static Linux x86_64 binary from the
[releases page](https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/releases),
check its checksum and extract it:

```bash
sha256sum -c energy-dashboard-for-kokkos-v0.3.0-x86_64-unknown-linux-musl.tar.gz.sha256
tar xzf energy-dashboard-for-kokkos-v0.3.0-x86_64-unknown-linux-musl.tar.gz
```

Or build from source:

```bash
cargo build --release
```

The resulting standalone binary is located at `target/release/energy-dashboard-for-kokkos`.

### 2. Generate a trace with the profiling connector

The KokkosP connector is in [`connector/`](connector): a single C++ file whose only
dependency is NVML. It writes the trace format described in [DATA_SPEC.md](DATA_SPEC.md).
CI builds it against an NVML stub, traces a simulated Kokkos run with it and analyzes the
result. The upstream pull requests
([#299](https://github.com/kokkos/kokkos-tools/pull/299),
[#301](https://github.com/kokkos/kokkos-tools/pull/301)) carry the earlier 2025 connector.

Build it with CMake (needs the CUDA toolkit for NVML):
```bash
cmake -S connector -B build-connector -DCMAKE_BUILD_TYPE=Release
cmake --build build-connector
# -> build-connector/libenergy_dashboard_connector.so
```

Traces written by the 2025 version of the connector, such as the ones published with the
[SMC 2025 poster](https://github.com/ethan-puyaubreau/smc2025-gpu-energy-poster#data-and-reproduction),
use an older CSV layout. Convert them with `analysis/to_trace_v1.py` from that repository, then
analyze the output directory as below.

### 3. Usage modes

#### Mode A: Direct Runner (Recommended)

Run and analyze your Kokkos application in a single step:

```bash
energy-dashboard-for-kokkos run --lib /path/to/libenergy_dashboard_connector.so --report report.html --perfetto trace.json -- ./my_app [args...]
```

The raw trace is written to a temporary directory and removed on exit. Pass
`--keep-trace <DIR>` to keep the CSV files for a later `analyze`. The exit code
of the application is forwarded, so batch jobs still see its failures.

#### Mode B: Post-Mortem Analysis

If the application was run independently:

```bash
# 1. Run with standard KokkosP environment variables
export KOKKOS_TOOLS_LIBS=/path/to/libenergy_dashboard_connector.so
export KOKKOS_TOOLS_OUTPUT_PATH=./my_trace
./my_app

# 2. Analyze the resulting directory
energy-dashboard-for-kokkos analyze ./my_trace --report report.html --perfetto trace.json
```

Output example (rows below 0.1% trimmed):

```text
  energy-dashboard-for-kokkos - App: energy_bench (Host: wsl-rtx3080ti, Backend: CUDA)
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

  Note: 100.0% of events are shorter than 100 ms (sampling every 20.4 ms, NVML refresh about 100 ms), their power is interpolated between readings rather than measured.
```

---

## Reading the report

- **Energy Incl** is the energy spent while a block was active, children included.
- **Energy Self** is the energy spent in the block itself, children excluded. The
  `% Self` column is based on it, so percentages sum to 100% with the idle row.
- **Idle (outside events)** is the energy measured while no instrumented block was
  running, including before the first and after the last event.
- **Avg Power** is the inclusive energy divided by the block duration.
- The lines under the table give the energy of each measured device. The table
  sums all of them.
- A final note appears when events are shorter than the power readings resolve (the
  sampling period or the NVML refresh, whichever is longer), with the share of such
  events. See Limits below.

### Attribution rules

- Each power series (one per domain and device) is integrated on its own with the
  trapezoidal rule, then the series are summed.
- Power is linearly interpolated at block boundaries.
- Sampled power is the only reference. Hardware cumulative energy counters are
  ignored: the analysis relies on sampled power only, so one method applies to every device.
- Blocks that overlap without being nested, such as concurrent kernels, share the
  energy of the overlapping interval equally.

### Multi-rank traces

Under MPI or Slurm the connector writes one `rank_<N>` subdirectory per rank.
`analyze` and `run` detect them and print one report per rank, in rank order.
Exports are written once per rank with the rank appended to the file name, for
instance `--report report.html` produces `report_rank_0.html`, `report_rank_1.html`.

Each rank samples every GPU visible on its node. Ranks sharing a node therefore
report the same device energy: do not sum device or total energies across ranks.

### Limits

- The connector samples power every 20 ms, but NVML itself refreshes the power
  reading only about every 100 ms, from the last 25 ms of each interval. Those figures
  were measured on A100 and H100 GPUs (Yang, Adamek and Armour, SC24); other GPUs may
  refresh differently, and the 100 ms floor is applied to every GPU trace. Kernels
  shorter than that are not measured individually: their power is interpolated
  between samples, and summing many launches only helps when they do not recur at
  the same phase as the sensor window. Regions much longer than the refresh interval,
  such as a solver phase or a whole algorithm, are measured reliably. The report
  prints the share of events shorter than about 100 ms so this is never silent.
- Overlapping blocks share energy equally because a device reports a single power
  value. With the default Kokkos global fencing, kernels do not overlap and this
  rule never applies.

---

## Visualizing traces

### 1. Interactive Perfetto timeline
Pass `--perfetto trace.json` and open [ui.perfetto.dev](https://ui.perfetto.dev). Drag-and-drop the JSON file to navigate slices with `W`, `A`, `S`, `D`. Nested blocks are stacked on a single track. Blocks that overlap without being nested are moved to an extra track, and each device gets its own power counter.

### 2. Standalone HTML report
Pass `--report report.html` and open the generated file directly in any web browser.

---

## Data specification

See [DATA_SPEC.md](DATA_SPEC.md) for complete details on the underlying `events.csv`, `power_samples.csv`, and `metadata.json` schemas.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup, conventions and
release process.

## Citing

If this tool supports published work, please cite it with the metadata in
[CITATION.cff](CITATION.cff) (GitHub shows it under "Cite this repository"). The
measurement limits and the DBSCAN case study are described in the
[SMC 2025 poster](https://ethan-puyaubreau.github.io/smc2025-gpu-energy-poster/).
