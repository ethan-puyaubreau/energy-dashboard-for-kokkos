# Contributing to energy-for-kokkos

Thanks for your interest in improving `energy-for-kokkos`. This guide covers the scope of
the project, how to set up a development environment and the conventions a pull
request is expected to follow.

## Scope and Settled Decisions

Please read this section before proposing a change to how energy is measured or
attributed.

- **NVIDIA GPUs only.** Other devices are out of scope for now. The trace format
  already accepts other domains, but no connector samples them.
- **Sampled power is the only reference.** The NVML cumulative energy counter gave
  unreliable measurements in practice. The trace format has no energy counter
  column on purpose, please do not reintroduce one.
- **The connector lives elsewhere.** The KokkosP connector that writes the traces is
  maintained in the
  [`kokkos-tools` fork](https://github.com/ethan-puyaubreau/kokkos-tools/tree/feat/v1-energy-profiler).
  Report connector issues there. This repository only reads traces.

The attribution rules are described in the README under "Reading the Report" and the
input format in [DATA_SPEC.md](DATA_SPEC.md). A change to either must update both the
code and the document.

## Development Setup

Requirements: Rust 1.88 or newer. No GPU is needed to work on the analysis tool, the
test fixtures contain recorded traces.

```bash
git clone https://github.com/ethan-puyaubreau/energy-for-kokkos.git
cd energy-for-kokkos
cargo build
```

Run the same checks as CI before pushing:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Try the tool on a recorded trace:

```bash
cargo run -- analyze tests/fixtures/real_rtx3080ti_trace --report report.html
```

## Code Map

| Module | Role |
| :--- | :--- |
| `src/model` | In-memory trace: events, power samples grouped per device, metadata |
| `src/parser` | Loading and validation of trace directories and rank subdirectories |
| `src/engine` | Power integration, energy attribution and the `run` command |
| `src/report` | Terminal table, Perfetto export and HTML report |
| `src/lib.rs` | Analysis and export pipeline shared by both commands |
| `tests/` | Integration tests on the fixtures in `tests/fixtures` |

## Conventions

### Code

- Formatting is done by `cargo fmt` only, never by hand.
- Every public item has a `///` doc comment, enforced by `missing_docs` in CI.
- Comments and docs use plain ASCII.
- Non-trivial logic comes with a test. Unit tests live next to the code, end to end
  checks go in `tests/fixtures.rs`.
- A new fixture is a directory under `tests/fixtures` following
  [DATA_SPEC.md](DATA_SPEC.md). Keep synthetic fixtures small enough that expected
  values can be computed by hand.

### Commits

Pull requests are merged with rebase, so every commit lands on `main` as is.

- One logical change per commit. Split refactoring, fixes and features.
- Every commit builds and passes the tests.
- Messages follow [Conventional Commits](https://www.conventionalcommits.org/) on a
  single line, with a scope when it helps, for instance
  `fix(engine): interpolate power at window boundaries`.
- No commit body unless the change really needs one.

### Changelog

Add a line under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md) for any change a
user can notice, in the matching `Added`, `Changed`, `Removed` or `Fixed` subsection.
Internal refactoring does not need an entry.

## Pull Requests

1. Open an issue first for anything larger than a bug fix, so the approach can be
   agreed before you write the code.
2. Branch from `main`.
3. Run the local checks above.
4. Open the pull request and fill in the template.
5. CI must be green before merging.

## Reporting Bugs

Use the bug report template. An attribution bug can rarely be reproduced without the
trace, so attach the trace directory (`events.csv`, `power_samples.csv`,
`metadata.json`) as a zip archive when you can. `energy-for-kokkos run --keep-trace <DIR>`
keeps it for you.

## Releasing

For maintainers only.

1. Bump `version` in `Cargo.toml`.
2. Rename `## [Unreleased]` in `CHANGELOG.md` to the new version and date, add a new
   empty `Unreleased` section and update the links at the bottom.
3. Merge to `main`, then tag and push:

   ```bash
   git tag -a vX.Y.Z -m "energy-for-kokkos X.Y.Z"
   git push origin vX.Y.Z
   ```

The release workflow checks that the tag matches the crate version, builds the static
Linux binary and publishes the GitHub release with the changelog section as notes.
