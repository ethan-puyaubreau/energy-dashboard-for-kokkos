use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

use crate::engine;
use crate::parser;
use crate::report;

/// Run a command instrumented with the Kokkos energy connector library.
pub fn run_instrumented_command(
    lib_path: &Path,
    app_args: &[String],
    perfetto: Option<&Path>,
    report_html: Option<&Path>,
) -> Result<()> {
    if app_args.is_empty() {
        anyhow::bail!(
            "No command specified to run. Usage: kokkos-energy run --lib <LIB> -- <APP> [ARGS...]"
        );
    }

    let tmp_dir = tempdir().context("Failed to create temporary trace directory")?;
    let trace_path = tmp_dir.path();

    let exe = &app_args[0];
    let args = &app_args[1..];

    println!(
        "  [kokkos-energy] Launching instrumented application: {}",
        exe
    );
    println!("  [kokkos-energy] Using connector: {}", lib_path.display());

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

    println!("\n  [kokkos-energy] Application finished. Analyzing trace...");
    let trace = parser::load_trace_dir(trace_path)?;
    let analysis = engine::analyze_trace(&trace);

    // Display terminal table
    report::print_terminal_report(&trace, &analysis);

    // Optional exports
    if let Some(perfetto_path) = perfetto {
        report::export_perfetto_trace(&trace, perfetto_path)?;
        println!("  Exported Perfetto trace to: {}", perfetto_path.display());
    }

    if let Some(html_path) = report_html {
        report::export_html_report(&trace, &analysis, html_path)?;
        println!(
            "  Exported interactive HTML report to: {}",
            html_path.display()
        );
    }

    Ok(())
}
