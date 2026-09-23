pub mod engine;
pub mod model;
pub mod parser;
pub mod report;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kokkos-energy")]
#[command(author = "Ethan Puyaubreau <ethan.puyaubreau@gmail.com>")]
#[command(version, about = "HPC energy analysis and profiling tool for Kokkos applications", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze an existing execution trace directory
    Analyze {
        /// Path to the directory containing events.csv and power_samples.csv
        #[arg(value_name = "TRACE_DIR")]
        trace_dir: PathBuf,

        /// Optional path to export a Perfetto/Chrome-Tracing JSON trace
        #[arg(short, long, value_name = "PERFETTO_FILE")]
        perfetto: Option<PathBuf>,

        /// Optional path to export a standalone interactive HTML dashboard
        #[arg(short, long, value_name = "HTML_FILE")]
        report: Option<PathBuf>,
    },

    /// Run an application with Kokkos energy profiling and analyze output immediately
    Run {
        /// Path to the energy profiler library (e.g. libkokkos_energy.so)
        #[arg(short, long, value_name = "LIB_PATH", env = "KOKKOS_TOOLS_LIBS")]
        lib: PathBuf,

        /// Optional path to export a Perfetto/Chrome-Tracing JSON trace
        #[arg(short, long, value_name = "PERFETTO_FILE")]
        perfetto: Option<PathBuf>,

        /// Optional path to export a standalone interactive HTML dashboard
        #[arg(short, long, value_name = "HTML_FILE")]
        report: Option<PathBuf>,

        /// Application executable and arguments
        #[arg(last = true, required = true, value_name = "COMMAND")]
        app_command: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            trace_dir,
            perfetto,
            report,
        } => {
            let trace = parser::load_trace_dir(&trace_dir)?;
            let analysis = engine::analyze_trace(&trace);

            // Display terminal table
            report::print_terminal_report(&trace, &analysis);

            // Export to Perfetto trace if requested
            if let Some(perfetto_path) = perfetto {
                report::export_perfetto_trace(&trace, &perfetto_path)?;
                println!("  Exported Perfetto trace to: {}", perfetto_path.display());
                println!("  Open https://ui.perfetto.dev to visualize the timeline.\n");
            }

            // Export to interactive HTML report if requested
            if let Some(html_path) = report {
                report::export_html_report(&trace, &analysis, &html_path)?;
                println!(
                    "  Exported interactive HTML report to: {}\n",
                    html_path.display()
                );
            }
        }

        Commands::Run {
            lib,
            perfetto,
            report,
            app_command,
        } => {
            engine::run_instrumented_command(
                &lib,
                &app_command,
                perfetto.as_deref(),
                report.as_deref(),
            )?;
        }
    }

    Ok(())
}
