pub mod events_csv;
pub mod power_csv;

use std::fs::File;
use std::path::Path;
use anyhow::{Context, Result};
use crate::model::{Metadata, Trace};

pub use events_csv::parse_events_csv;
pub use power_csv::parse_power_csv;

/// Ingest an entire trace directory.
pub fn load_trace_dir<P: AsRef<Path>>(dir: P) -> Result<Trace> {
    let dir = dir.as_ref();
    let events_path = dir.join("events.csv");
    let power_path = dir.join("power_samples.csv");
    let meta_path = dir.join("metadata.json");

    let events = parse_events_csv(&events_path)?;
    let samples = parse_power_csv(&power_path)?;

    let metadata = if meta_path.exists() {
        let file = File::open(&meta_path)
            .with_context(|| format!("Failed to open metadata file: {}", meta_path.display()))?;
        let meta: Metadata = serde_json::from_reader(file)
            .with_context(|| format!("Malformed JSON in metadata file: {}", meta_path.display()))?;
        Some(meta)
    } else {
        None
    };

    Ok(Trace::new(metadata, events, samples))
}
