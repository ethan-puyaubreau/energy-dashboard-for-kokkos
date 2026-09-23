pub mod engine;
pub mod model;
pub mod parser;
pub mod report;

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};

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
    /// Analyze an execution trace directory
    Analyze {
        /// Path to the directory containing events.csv and power_samples.csv
        #[arg(value_name = "TRACE_DIR")]
        trace_dir: PathBuf,

        /// Optional path to export a Perfetto/Chrome-Tracing JSON trace
        #[arg(short, long, value_name = "PERFETTO_FILE")]
        perfetto: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { trace_dir, perfetto } => {
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
        }
    }

    Ok(())
}
