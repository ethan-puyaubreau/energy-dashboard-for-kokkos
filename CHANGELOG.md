# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-09-24

### Changed

- The package and the binary are renamed from `kokkos-energy` to
  `energy-dashboard-for-kokkos`, the name of the repository, so that the tool is not
  mistaken for an official Kokkos project. Release archives follow the new name.
  Scripts calling `kokkos-energy` need the new binary name; the analysis, the trace
  format and the command line options are unchanged.
- The README states the NVML refresh limit: kernels shorter than about 100 ms are not
  measured individually, longer regions are.

### Added

- `CITATION.cff` with the software metadata.

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
