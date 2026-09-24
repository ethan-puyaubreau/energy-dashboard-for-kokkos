pub mod events_csv;
pub mod power_csv;

use crate::model::{Metadata, Trace};
use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

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

    // An empty trace would silently report zero Joules
    if events.is_empty() {
        anyhow::bail!("No events recorded in {}", events_path.display());
    }
    if samples.is_empty() {
        anyhow::bail!("No power samples recorded in {}", power_path.display());
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_trace_without_samples() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("events.csv"),
            "id,parent_id,name,category,start_ns,end_ns\n1,0,Main,USER_REGION,100,500\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("power_samples.csv"),
            "timestamp_ns,domain,device_id,power_watts,energy_joules\n",
        )
        .unwrap();
        assert!(load_trace_dir(dir.path()).is_err());
    }
}
