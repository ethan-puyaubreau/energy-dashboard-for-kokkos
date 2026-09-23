pub mod perfetto;
pub mod terminal;

pub use perfetto::export_perfetto_trace;
pub use terminal::print_terminal_report;
