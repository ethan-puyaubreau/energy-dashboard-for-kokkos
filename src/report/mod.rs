pub mod html;
pub mod perfetto;
pub mod terminal;

pub use html::export_html_report;
pub use perfetto::export_perfetto_trace;
pub use terminal::{print_terminal_report, sampling_note};
