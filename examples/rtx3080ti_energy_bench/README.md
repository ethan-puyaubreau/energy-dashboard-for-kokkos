One run of `energy_bench` from [`benchmark/`](../../benchmark), recorded on an NVIDIA
RTX 3080 Ti under WSL2 on 2026-09-23 with an early version of the connector
([ethan-puyaubreau/kokkos-tools@5f8b1e9](https://github.com/ethan-puyaubreau/kokkos-tools/commit/5f8b1e9)).
It slept 20 ms between power readings, hence the 20.4 ms sampling period in the report.

The benchmark runs `daxpy` for 2 s outside any region as a warmup, then loops on `matvec`
inside the `MatVec` region and on `reduce` inside the `Reduction` region, about 4 s each.
Almost every kernel is shorter than the NVML refresh of about 100 ms, so the report adds a
note: their power is interpolated between readings rather than measured.
