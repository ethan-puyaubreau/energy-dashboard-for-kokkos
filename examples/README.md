# Example traces

Two traces recorded on real GPUs, in the trace format v1.0 described in
[DATA_SPEC.md](../DATA_SPEC.md). They need no GPU to analyze:

```bash
energy-dashboard-for-kokkos analyze examples/h100_arborx_fdbscan
```

| Directory | GPU | Application | Events | Duration | What it shows |
|-----------|-----|-------------|-------:|---------:|---------------|
| [`h100_arborx_fdbscan`](h100_arborx_fdbscan) | H100 NVL | ArborX DBSCAN (fdbscan) | 29 | 10.2 s | A real application: one traversal kernel and the idle time each take almost half of the energy |
| [`rtx3080ti_energy_bench`](rtx3080ti_energy_bench) | RTX 3080 Ti | `energy_bench` from [`benchmark/`](../benchmark) | 56,768 | 10.0 s | Thousands of kernels too short for the power readings, which the report flags |

The regression tests in `tests/fixtures.rs` load both: they pin the DBSCAN figures of the H100
trace, and check on the RTX trace that every Joule is attributed exactly once.
