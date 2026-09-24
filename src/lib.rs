pub mod engine;
pub mod model;
pub mod parser;
pub mod report;

use anyhow::Result;
use std::path::Path;

/// Analyze a trace directory, print the terminal report and write the requested exports.
pub fn analyze_and_report(
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
