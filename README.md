# kokkos-energy

High-performance energy analysis and profiling tool for Kokkos applications.

`kokkos-energy` provides attribution of electrical energy (Joules) and average power (Watts) to Kokkos execution blocks (`UserRegion`, `parallel_for`, `parallel_reduce`, `parallel_scan`, and memory movements).

## Key Features

- **Zero-Daemon, Zero-Docker:** Single native binary suitable for unprivileged HPC environments.
- **Hierarchical Energy Attribution:** Accurately attributes energy across enclosing regions and kernels using trapezoidal numerical integration.
- **HPC Console Report:** Formatted summary table printed directly to stdout (ideal for Slurm batch job logs).
- **Perfetto / Chrome Tracing Export:** Generate interactive `trace.json` timelines viewable at [ui.perfetto.dev](https://ui.perfetto.dev).

---

## Quick Start

### 1. Build from source

```bash
cargo build --release
```

The resulting standalone binary is located at `target/release/kokkos-energy`.

### 2. Analyze a trace directory

```bash
kokkos-energy analyze ./path/to/trace_dir
```

Output example:

```text
  Kokkos Energy Analysis - App: synthetic_bench (Host: test-node, Backend: CUDA)
┌──────────────────────┬─────────────────┬───────┬──────────────┬────────────┬───────────────┬──────────┐
│ Block / Kernel       ┆ Category        ┆ Calls ┆ Duration (s) ┆ Energy (J) ┆ Avg Power (W) ┆ % Energy │
╞══════════════════════╪═════════════════╪═══════╪══════════════╪════════════╪═══════════════╪══════════╡
│ MainLoop             ┆ USER_REGION     ┆ 1     ┆ 8.000        ┆ 1875.00    ┆ 234.4         ┆ 100.0%   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Reduction            ┆ PARALLEL_REDUCE ┆ 1     ┆ 3.000        ┆ 900.00     ┆ 300.0         ┆ 48.0%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ MatVec               ┆ PARALLEL_FOR    ┆ 1     ┆ 3.000        ┆ 600.00     ┆ 200.0         ┆ 32.0%    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Total Trace (Active) ┆ -               ┆ -     ┆ 8.000        ┆ 1875.00    ┆ 234.4         ┆ 100.0%   │
└──────────────────────┴─────────────────┴───────┴──────────────┴────────────┴───────────────┴──────────┘
```

### 3. Export to Perfetto Timeline

```bash
kokkos-energy analyze ./path/to/trace_dir --perfetto trace.json
```

Then open [https://ui.perfetto.dev](https://ui.perfetto.dev) and drag-and-drop `trace.json` to navigate through execution slices and power curves with keyboard shortcuts (`W`, `A`, `S`, `D`).

---

## Trace Data Specification

See [DATA_SPEC.md](DATA_SPEC.md) for complete details on `events.csv`, `power_samples.csv`, and `metadata.json`.
