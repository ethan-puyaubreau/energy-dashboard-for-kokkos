use anyhow::{Context, Result};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::model::{Event, Trace};

/// Assign each event to a track so that events sharing a track are properly nested.
///
/// Perfetto draws the nested slices of one track as a call stack, but slices that
/// overlap without being nested are rendered incorrectly. Such events are moved to
/// the first track whose open slices can contain them.
/// `events` must be sorted by start timestamp, parents first.
fn assign_tracks(events: &[Event]) -> Vec<usize> {
    // End timestamps of the open slices of each track, innermost last
    let mut stacks: Vec<Vec<u64>> = Vec::new();

    events
        .iter()
        .map(|e| {
            let end_ns = e.end_ns.max(e.start_ns);
            let track = stacks.iter_mut().position(|stack| {
                while stack.last().is_some_and(|&top| top <= e.start_ns) {
                    stack.pop();
                }
                stack.last().is_none_or(|&top| end_ns <= top)
            });

            let track = track.unwrap_or_else(|| {
                stacks.push(Vec::new());
                stacks.len() - 1
            });
            stacks[track].push(end_ns);
            track
        })
        .collect()
}

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

    // 1. Regions and Kernels as Complete Events (type 'X')
    let tracks = assign_tracks(&trace.events);
    for (e, track) in trace.events.iter().zip(tracks) {
        let ts_us = (e.start_ns.saturating_sub(min_ts_ns)) as f64 / 1_000.0;
        let dur_us = e.duration_ns() as f64 / 1_000.0;

        trace_events.push(json!({
            "name": e.name,
            "cat": e.category.to_string(),
            "ph": "X",
            "ts": ts_us,
            "dur": dur_us,
            "pid": 1,
            "tid": track,
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
    use crate::model::RegionCategory;

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
    fn test_nested_events_share_a_track() {
        let events = vec![event(1, 0, 0, 10), event(2, 1, 1, 4), event(3, 1, 5, 9)];
        assert_eq!(assign_tracks(&events), vec![0, 0, 0]);
    }

    #[test]
    fn test_overlapping_siblings_get_separate_tracks() {
        let events = vec![event(1, 0, 0, 10), event(2, 1, 1, 6), event(3, 1, 4, 9)];
        assert_eq!(assign_tracks(&events), vec![0, 0, 1]);
    }

    #[test]
    fn test_exported_tid_is_the_track() {
        let events = vec![event(1, 0, 0, 2), event(2, 0, 1, 3)];
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
        assert_eq!(tids, vec![0, 1]);
    }
}
