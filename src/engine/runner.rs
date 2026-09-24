use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

use crate::analyze_and_report;

/// Run a command instrumented with the Kokkos energy connector library.
///
/// Returns the exit code of the application so that batch jobs see its failures.
/// The trace of a failed application is still analyzed when it can be loaded.
/// Raw trace files are kept in `keep_trace` when given, otherwise they are
/// written to a temporary directory removed on exit.
pub fn run_instrumented_command(
    lib_path: &Path,
    app_args: &[String],
    keep_trace: Option<&Path>,
    perfetto: Option<&Path>,
    report_html: Option<&Path>,
) -> Result<i32> {
    if app_args.is_empty() {
        anyhow::bail!(
            "No command specified to run. Usage: kokkos-energy run --lib <LIB> -- <APP> [ARGS...]"
        );
    }

    let tmp_dir;
    let trace_path = match keep_trace {
        Some(dir) => {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create trace directory: {}", dir.display()))?;
            dir
        }
        None => {
            tmp_dir = tempdir().context("Failed to create temporary trace directory")?;
            tmp_dir.path()
        }
    };

    let exe = &app_args[0];
    let args = &app_args[1..];

    println!(
        "  [kokkos-energy] Launching instrumented application: {}",
        exe
    );
    println!("  [kokkos-energy] Using connector: {}", lib_path.display());
    if let Some(dir) = keep_trace {
        println!("  [kokkos-energy] Keeping raw trace in: {}", dir.display());
    }

    let mut cmd = Command::new(exe);
    cmd.args(args);
    cmd.env("KOKKOS_TOOLS_LIBS", lib_path);
    cmd.env("KOKKOS_TOOLS_OUTPUT_PATH", trace_path);

    let status = cmd
        .status()
        .with_context(|| format!("Failed to execute process: {}", exe))?;

    if !status.success() {
        eprintln!(
            "  [kokkos-energy] Warning: application exited with status: {}",
            status
        );
    }

    // A process killed by a signal has no exit code
    let code = status.code().unwrap_or(1);

    println!("\n  [kokkos-energy] Application finished. Analyzing trace...");
    match analyze_and_report(trace_path, perfetto, report_html) {
        Ok(()) => Ok(code),
        Err(err) if code != 0 => {
            eprintln!("  [kokkos-energy] Could not analyze trace: {err:#}");
            Ok(code)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_exit_code_is_returned() {
        let app = if cfg!(windows) {
            ["cmd", "/C", "exit 3"]
        } else {
            ["sh", "-c", "exit 3"]
        }
        .map(String::from);
        let code =
            run_instrumented_command(Path::new("unused.so"), &app, None, None, None).unwrap();
        assert_eq!(code, 3);
    }
}
