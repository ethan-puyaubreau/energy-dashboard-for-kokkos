use anyhow::{Context, Result};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::model::Trace;

/// Export the trace to Chrome Tracing / Perfetto JSON format.
///
/// Can be loaded directly into https://ui.perfetto.dev or chrome://tracing.
pub fn export_perfetto_trace<P: AsRef<Path>>(trace: &Trace, out_path: P) -> Result<()> {
    let out_path = out_path.as_ref();
    let mut file = File::create(out_path).with_context(|| {
        format!(
            "Failed to create Perfetto trace file: {}",
            out_path.display()
        )
    })?;

    // Determine baseline timestamp (microseconds)
    let min_ts_ns = trace
        .events
        .iter()
        .map(|e| e.start_ns)
        .chain(trace.samples.iter().map(|s| s.timestamp_ns))
        .min()
        .unwrap_or(0);

    let mut trace_events = Vec::new();

    // 1. Regions and Kernels as Complete Events (type 'X')
    for e in &trace.events {
        let ts_us = (e.start_ns.saturating_sub(min_ts_ns)) as f64 / 1_000.0;
        let dur_us = e.duration_ns() as f64 / 1_000.0;

        trace_events.push(json!({
            "name": e.name,
            "cat": e.category.to_string(),
            "ph": "X",
            "ts": ts_us,
            "dur": dur_us,
            "pid": 1,
            "tid": e.parent_id,
            "args": {
                "id": e.id,
                "parent_id": e.parent_id
            }
        }));
    }

    // 2. Power samples as Counter Events (type 'C')
    for s in &trace.samples {
        let ts_us = (s.timestamp_ns.saturating_sub(min_ts_ns)) as f64 / 1_000.0;
        let counter_name = format!("{}_{}_power_watts", s.domain, s.device_id);

        trace_events.push(json!({
            "name": counter_name,
            "cat": "power",
            "ph": "C",
            "ts": ts_us,
            "pid": 1,
            "args": {
                "Watts": s.power_watts
            }
        }));
    }

    let output = json!({
        "traceEvents": trace_events,
        "displayTimeUnit": "ms"
    });

    serde_json::to_writer_pretty(&mut file, &output)
        .with_context(|| "Failed to serialize Perfetto JSON")?;
    file.flush()?;

    Ok(())
}
