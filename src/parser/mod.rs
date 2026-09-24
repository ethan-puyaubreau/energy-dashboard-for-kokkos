pub mod events_csv;
pub mod power_csv;

use crate::model::{Metadata, Trace};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub use events_csv::parse_events_csv;
pub use power_csv::parse_power_csv;

/// Resolve the directory that holds the trace files.
///
/// Under MPI or Slurm the connector writes into a `rank_<N>` subdirectory.
/// A single rank subdirectory is used transparently, several are rejected.
fn resolve_trace_dir(dir: &Path) -> Result<PathBuf> {
    if dir.join("events.csv").exists() {
        return Ok(dir.to_path_buf());
    }

    let ranks: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let is_rank = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("rank_"));
            is_rank && path.join("events.csv").exists()
        })
        .collect();

    match ranks.as_slice() {
        [single] => Ok(single.clone()),
        [] => Ok(dir.to_path_buf()),
        _ => anyhow::bail!(
            "Found {} rank directories in {}, analyze one rank directory at a time",
            ranks.len(),
            dir.display()
        ),
    }
}

/// Ingest an entire trace directory.
pub fn load_trace_dir<P: AsRef<Path>>(dir: P) -> Result<Trace> {
    let dir = resolve_trace_dir(dir.as_ref())?;
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
    fn test_single_rank_subdirectory_is_resolved() {
        let dir = tempfile::tempdir().unwrap();
        let rank = dir.path().join("rank_0");
        fs::create_dir(&rank).unwrap();
        fs::write(rank.join("events.csv"), "").unwrap();
        assert_eq!(resolve_trace_dir(dir.path()).unwrap(), rank);

        fs::create_dir(dir.path().join("rank_1")).unwrap();
        fs::write(dir.path().join("rank_1").join("events.csv"), "").unwrap();
        assert!(resolve_trace_dir(dir.path()).is_err());
    }

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
