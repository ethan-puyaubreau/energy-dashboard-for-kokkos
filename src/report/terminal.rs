//! Summary table printed to standard output.

use std::fmt::Write;

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};

use crate::engine::TraceAnalysis;
use crate::model::Trace;

/// Describe how many events are too short to be measured by the power sampler.
///
/// Returns `None` when every event lasts at least as long as the power readings resolve.
pub fn sampling_note(analysis: &TraceAnalysis) -> Option<String> {
    let period = analysis.sampling_period_sec?;
    let resolution = analysis.resolution_sec?;
    if analysis.short_event_fraction <= 0.0 {
        return None;
    }

    let limit = if resolution > period {
        format!(
            "{:.0} ms (sampling every {:.1} ms, NVML refresh about 100 ms)",
            resolution * 1_000.0,
            period * 1_000.0
        )
    } else {
        format!("the {:.1} ms sampling period", period * 1_000.0)
    };
    // Round down, so a few long events never show as "100.0%".
    let percent = (analysis.short_event_fraction * 1_000.0).floor() / 10.0;
    Some(format!(
        "{percent:.1}% of events are shorter than {limit}, their power is interpolated \
         between readings rather than measured."
    ))
}

/// Print summary table and metadata to standard output.
pub fn print_terminal_report(trace: &Trace, analysis: &TraceAnalysis) {
    print!("{}", render_terminal_report(trace, analysis));
}

/// Build the text printed by [`print_terminal_report`].
pub fn render_terminal_report(trace: &Trace, analysis: &TraceAnalysis) -> String {
    // Writing to a String cannot fail, so the results of writeln! are ignored.
    let mut out = String::from("\n");
    if let Some(ref meta) = trace.metadata {
        let app = meta.app_name.as_deref().unwrap_or("Unknown");
        let host = meta.hostname.as_deref().unwrap_or("Unknown");
        let backend = meta.kokkos_backend.as_deref().unwrap_or("Unknown");
        let _ = writeln!(
            out,
            "  energy-dashboard-for-kokkos - App: {} (Host: {}, Backend: {})",
            app, host, backend
        );
    } else {
        out.push_str("  energy-dashboard-for-kokkos report\n");
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        Cell::new("Block / Kernel").fg(Color::Cyan),
        Cell::new("Category").fg(Color::Cyan),
        Cell::new("Calls").fg(Color::Cyan),
        Cell::new("Duration (s)").fg(Color::Cyan),
        Cell::new("Energy Incl (J)").fg(Color::Cyan),
        Cell::new("Energy Self (J)").fg(Color::Cyan),
        Cell::new("Avg Power (W)").fg(Color::Cyan),
        Cell::new("% Self").fg(Color::Cyan),
    ]);

    for r in &analysis.regions {
        table.add_row(Row::from(vec![
            Cell::new(&r.name),
            Cell::new(r.category.to_string()),
            Cell::new(r.call_count.to_string()),
            Cell::new(format!("{:.3}", r.total_duration_sec)),
            Cell::new(format!("{:.2}", r.inclusive_energy_joules)),
            Cell::new(format!("{:.2}", r.exclusive_energy_joules)),
            Cell::new(format!("{:.1}", r.avg_power_watts)),
            Cell::new(format!("{:.1}%", r.energy_percentage)),
        ]));
    }

    // Energy measured while no event was running
    let idle_pct = if analysis.total_trace_energy_joules > 0.0 {
        analysis.idle_energy_joules / analysis.total_trace_energy_joules * 100.0
    } else {
        0.0
    };
    let idle_power = if analysis.idle_duration_sec > 0.0 {
        analysis.idle_energy_joules / analysis.idle_duration_sec
    } else {
        0.0
    };
    table.add_row(Row::from(vec![
        Cell::new("Idle (outside events)").fg(Color::DarkGrey),
        Cell::new("-"),
        Cell::new("-"),
        Cell::new(format!("{:.3}", analysis.idle_duration_sec)),
        Cell::new(format!("{:.2}", analysis.idle_energy_joules)),
        Cell::new(format!("{:.2}", analysis.idle_energy_joules)),
        Cell::new(format!("{:.1}", idle_power)),
        Cell::new(format!("{:.1}%", idle_pct)),
    ]));

    // Total trace summary row
    table.add_row(Row::from(vec![
        Cell::new("Total Trace").fg(Color::Yellow),
        Cell::new("-"),
        Cell::new("-"),
        Cell::new(format!("{:.3}", analysis.total_trace_duration_sec)).fg(Color::Yellow),
        Cell::new(format!("{:.2}", analysis.total_trace_energy_joules)).fg(Color::Yellow),
        Cell::new(format!("{:.2}", analysis.total_trace_energy_joules)).fg(Color::Yellow),
        Cell::new(format!("{:.1}", analysis.avg_trace_power_watts)).fg(Color::Yellow),
        Cell::new("100.0%").fg(Color::Yellow),
    ]));

    let _ = writeln!(out, "{table}");

    // Energy per measured device, the table above sums all of them
    for d in &analysis.devices {
        let _ = writeln!(
            out,
            "  {} {}: {:.2} J, {:.1} W avg",
            d.domain, d.device_id, d.energy_joules, d.avg_power_watts
        );
    }

    if let Some(note) = sampling_note(analysis) {
        let _ = writeln!(out, "\n  Note: {note}");
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis(period: f64, resolution: f64) -> TraceAnalysis {
        TraceAnalysis {
            sampling_period_sec: Some(period),
            resolution_sec: Some(resolution),
            short_event_fraction: 0.5,
            ..TraceAnalysis::default()
        }
    }

    #[test]
    fn note_names_nvml_only_when_it_sets_the_limit() {
        let gpu = sampling_note(&analysis(0.02, 0.1)).unwrap();
        assert!(gpu.contains("shorter than 100 ms (sampling every 20.0 ms, NVML refresh"));

        let cpu = sampling_note(&analysis(0.02, 0.02)).unwrap();
        assert!(cpu.contains("shorter than the 20.0 ms sampling period"));
        assert!(!cpu.contains("NVML"));
    }
}
