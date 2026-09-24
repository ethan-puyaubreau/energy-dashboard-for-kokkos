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
    let min_ts_ns = trace.time_bounds().map_or(0, |(min, _)| min);

    let mut trace_events = Vec::new();

    // 1. Regions and Kernels as Complete Events (type 'X'), one track per nesting depth
    let depths = trace.event_depths();
    for (e, depth) in trace.events.iter().zip(depths) {
        let ts_us = (e.start_ns.saturating_sub(min_ts_ns)) as f64 / 1_000.0;
        let dur_us = e.duration_ns() as f64 / 1_000.0;

        trace_events.push(json!({
            "name": e.name,
            "cat": e.category.to_string(),
            "ph": "X",
            "ts": ts_us,
            "dur": dur_us,
            "pid": 1,
            "tid": depth,
            "args": {
                "id": e.id,
                "parent_id": e.parent_id
            }
        }));
    }

    // 2. Power samples as Counter Events (type 'C')
    for s in trace.samples() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Event, RegionCategory};

    fn event(id: u64, parent_id: u64, start_ns: u64, end_ns: u64) -> Event {
        Event {
            id,
            parent_id,
            name: format!("e{id}"),
            category: RegionCategory::UserRegion,
            start_ns,
            end_ns,
        }
    }

    #[test]
    fn test_tracks_follow_nesting_depth() {
        // Two siblings under the same root must share a track
        let events = vec![event(1, 0, 0, 10), event(2, 1, 1, 4), event(3, 1, 5, 9)];
        let trace = Trace::new(None, events, Vec::new());
        let file = tempfile::NamedTempFile::new().unwrap();
        export_perfetto_trace(&trace, file.path()).unwrap();

        let json: serde_json::Value =
            serde_json::from_reader(File::open(file.path()).unwrap()).unwrap();
        let tids: Vec<u64> = json["traceEvents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["tid"].as_u64().unwrap())
            .collect();
        assert_eq!(tids, vec![0, 1, 1]);
    }
}
