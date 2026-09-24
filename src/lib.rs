pub mod engine;
pub mod model;
pub mod parser;
pub mod report;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Insert `suffix` between the stem and the extension of `path`.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let name = match path.extension() {
        Some(ext) => format!("{stem}_{suffix}.{}", ext.to_string_lossy()),
        None => format!("{stem}_{suffix}"),
    };
    path.with_file_name(name)
}

/// Analyze a trace directory, print the terminal report and write the requested exports.
///
/// A multi-rank trace is analyzed one rank at a time. Each rank gets its own report
/// and its exports are suffixed with the rank directory name, for instance
/// `report_rank_0.html`.
pub fn analyze_and_report(
    trace_dir: &Path,
    perfetto: Option<&Path>,
    report_html: Option<&Path>,
) -> Result<()> {
    let dirs = parser::trace_dirs(trace_dir);
    if dirs.len() == 1 {
        return analyze_single(&dirs[0], perfetto, report_html);
    }

    for dir in &dirs {
        let rank = dir.file_name().unwrap_or_default().to_string_lossy();
        println!("\n  ===== {rank} =====");
        analyze_single(
            dir,
            perfetto.map(|p| with_suffix(p, &rank)).as_deref(),
            report_html.map(|p| with_suffix(p, &rank)).as_deref(),
        )?;
    }
    Ok(())
}

/// Analyze the trace files of a single directory.
fn analyze_single(
    trace_dir: &Path,
    perfetto: Option<&Path>,
    report_html: Option<&Path>,
) -> Result<()> {
    let trace = parser::load_trace_dir(trace_dir)?;
    let analysis = engine::analyze_trace(&trace);

    report::print_terminal_report(&trace, &analysis);

    if let Some(perfetto_path) = perfetto {
        report::export_perfetto_trace(&trace, perfetto_path)?;
        println!("  Exported Perfetto trace to: {}", perfetto_path.display());
        println!("  Open https://ui.perfetto.dev to visualize the timeline.\n");
    }

    if let Some(html_path) = report_html {
        report::export_html_report(&trace, &analysis, html_path)?;
        println!(
            "  Exported interactive HTML report to: {}\n",
            html_path.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_paths_are_suffixed_with_rank() {
        assert_eq!(
            with_suffix(Path::new("out/report.html"), "rank_1"),
            Path::new("out/report_rank_1.html")
        );
        assert_eq!(
            with_suffix(Path::new("trace"), "rank_0"),
            Path::new("trace_rank_0")
        );
    }
}
