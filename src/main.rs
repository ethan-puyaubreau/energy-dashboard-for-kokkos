use anyhow::Result;
use clap::{Parser, Subcommand};
use energy_dashboard_for_kokkos::{analyze_and_report, engine};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "energy-dashboard-for-kokkos")]
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
        /// Path to the energy profiler library (e.g. libenergy_dashboard_connector.so)
        #[arg(short, long, value_name = "LIB_PATH", env = "KOKKOS_TOOLS_LIBS")]
        lib: PathBuf,

        /// Keep the raw trace files in this directory instead of a temporary one
        #[arg(short, long, value_name = "DIR")]
        keep_trace: Option<PathBuf>,

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
            analyze_and_report(&trace_dir, perfetto.as_deref(), report.as_deref())?;
        }

        Commands::Run {
            lib,
            keep_trace,
            perfetto,
            report,
            app_command,
        } => {
            let code = engine::run_instrumented_command(
                &lib,
                &app_command,
                keep_trace.as_deref(),
                perfetto.as_deref(),
                report.as_deref(),
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
    }

    Ok(())
}
