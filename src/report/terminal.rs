//! Summary table printed to standard output.

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};

use crate::engine::TraceAnalysis;
use crate::model::Trace;

/// Describe how many events are too short to be measured by the power sampler.
///
/// Returns `None` when every event spans at least one sampling period.
pub fn sampling_note(analysis: &TraceAnalysis) -> Option<String> {
    let period = analysis.sampling_period_sec?;
    if analysis.short_event_fraction <= 0.0 {
        return None;
    }

    Some(format!(
        "{:.1}% of events are shorter than the {:.1} ms sampling period, \
         their power is interpolated between samples rather than measured.",
        analysis.short_event_fraction * 100.0,
        period * 1_000.0
    ))
}

/// Print summary table and metadata to standard output.
pub fn print_terminal_report(trace: &Trace, analysis: &TraceAnalysis) {
    println!();
    if let Some(ref meta) = trace.metadata {
        let app = meta.app_name.as_deref().unwrap_or("Unknown");
        let host = meta.hostname.as_deref().unwrap_or("Unknown");
        let backend = meta.kokkos_backend.as_deref().unwrap_or("Unknown");
        println!(
            "  Kokkos Energy Analysis - App: {} (Host: {}, Backend: {})",
            app, host, backend
        );
    } else {
        println!("  Kokkos Energy Analysis Report");
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

    println!("{table}");

    // Energy per measured device, the table above sums all of them
    for d in &analysis.devices {
        println!(
            "  {} {}: {:.2} J, {:.1} W avg",
            d.domain, d.device_id, d.energy_joules, d.avg_power_watts
        );
    }

    if let Some(note) = sampling_note(analysis) {
        println!("\n  Note: {note}");
    }
    println!();
}
