# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Zenodo DOI (10.5281/zenodo.22943410, all versions) in the README and `CITATION.cff`.
- `device_count` and `mpi_rank`, written by the connector in `metadata.json`, are part of
  the trace format and read into the metadata.

### Changed

- The connector is no longer copied under `connector/`: the kokkos-tools fork
  (`profiling/energy-profiler`) is its only source, and CI builds a pinned commit of it.
  The library is now `libkp_energy_profiler.so`.

### Removed

- The `energy_joules` column of `power_samples.csv`. Traces that still carry it are read
  as before, the column was already ignored.

## [0.3.0] - 2026-09-24

### Added

- `CITATION.cff` with the software metadata.
- Zenodo metadata (`.zenodo.json`), so each release is archived with a DOI.
- The v1 KokkosP connector under `connector/` (`libenergy_dashboard_connector.so`), with a CMake build and a CI job that builds
  it against an NVML stub, traces a simulated Kokkos run and analyzes the trace.
- A regression test on one ArborX DBSCAN run traced on an H100 NVL for the SMC 2025 poster.

### Changed

- The package and the binary are renamed from `kokkos-energy` to
  `energy-dashboard-for-kokkos`, the name of the repository, so that the tool is not
  mistaken for an official Kokkos project. Release archives follow the new name.
  Scripts calling `kokkos-energy` need the new binary name; the analysis, the trace
  format and the command line options are unchanged.
- In GPU traces, events are flagged as not measured individually when shorter than the
  NVML refresh interval (about 100 ms), not only when shorter than the sampling period.
  CPU-only traces keep the sampling period. The README states that limit.
- The terminal and HTML report headers name the tool instead of "Kokkos Energy".

## [0.2.0] - 2026-09-24

### Added

- Inclusive and exclusive energy per block, `% Self` sums to 100% with the idle row.
- Idle energy measured outside instrumented events.
- Energy summary per measured device under the terminal table.
- Note when events are shorter than the power sampling period.
- One report per rank for multi-rank traces, exports suffixed with the rank.
- `run --keep-trace <DIR>` to keep the raw trace files.
- Prebuilt static Linux x86_64 binaries attached to GitHub releases.
- Requirements, attribution rules and limits in the README.

### Changed

- Each (domain, device) power series is integrated separately, then summed.
- Power is linearly interpolated at block boundaries.
- Overlapping blocks share the energy of the overlapping interval equally.
- The total trace window spans every event and every sample.
- The HTML report embeds Plotly and works without network access.
- The HTML report plots one power curve per device.
- Perfetto keeps nested blocks on one track and moves overlapping ones to extra tracks.
- `run` forwards the exit code of the application.
- Minimum supported Rust version is 1.88.

### Removed

- Use of hardware cumulative energy counters. The `energy_joules` column is still
  accepted but ignored, sampled power is the only reference.

### Fixed

- Energy of interleaved samples from different devices was integrated as a single
  series.
- Percentages of nested regions and kernels counted the same energy twice.
- Empty, non-finite or inconsistent trace files were silently accepted.
- Kernel and application names could break the HTML report.

## [0.1.0]

### Added

- `analyze` and `run` commands.
- Trapezoidal energy integration and attribution per region and kernel.
- Terminal report, Perfetto trace export and HTML report.

[Unreleased]: https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ethan-puyaubreau/energy-dashboard-for-kokkos/releases/tag/v0.2.0
