//! Energy integration, attribution and instrumented application runner.

pub mod attribution;
pub mod integrate;
pub mod runner;

pub use attribution::{analyze_trace, DeviceMetrics, RegionMetrics, TraceAnalysis};
pub use integrate::integrate_energy_joules;
pub use runner::run_instrumented_command;
