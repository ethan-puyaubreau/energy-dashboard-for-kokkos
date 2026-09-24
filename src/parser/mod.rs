//! Loading of trace directories written by the energy connector.

pub mod events_csv;
pub mod power_csv;

use crate::model::{Metadata, Trace};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub use events_csv::parse_events_csv;
pub use power_csv::parse_power_csv;

/// List the directories that hold trace files under `dir`.
///
/// Under MPI or Slurm the connector writes one `rank_<N>` subdirectory per rank.
/// Rank directories are returned in rank order. When `dir` holds the trace files
/// itself, or no rank directory is found, `dir` is returned alone.
pub fn trace_dirs(dir: &Path) -> Vec<PathBuf> {
    if dir.join("events.csv").exists() {
        return vec![dir.to_path_buf()];
    }

    let mut ranks: Vec<(u64, PathBuf)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("events.csv").exists())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let rank = name.strip_prefix("rank_")?.parse().ok()?;
            Some((rank, path))
        })
        .collect();
    ranks.sort();

    if ranks.is_empty() {
        vec![dir.to_path_buf()]
    } else {
        ranks.into_iter().map(|(_, path)| path).collect()
    }
}

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
    fn test_rank_subdirectories_are_listed_in_rank_order() {
        let dir = tempfile::tempdir().unwrap();
        for rank in ["rank_10", "rank_2", "other"] {
            fs::create_dir(dir.path().join(rank)).unwrap();
            fs::write(dir.path().join(rank).join("events.csv"), "").unwrap();
        }

        let dirs = trace_dirs(dir.path());
        assert_eq!(
            dirs,
            vec![dir.path().join("rank_2"), dir.path().join("rank_10")]
        );
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
            "timestamp_ns,domain,device_id,power_watts\n",
        )
        .unwrap();
        assert!(load_trace_dir(dir.path()).is_err());
    }

    #[test]
    fn test_metadata_written_by_the_connector() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("events.csv"),
            "id,parent_id,name,category,start_ns,end_ns\n1,0,Main,USER_REGION,100,500\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("power_samples.csv"),
            "timestamp_ns,domain,device_id,power_watts\n100,GPU,0,250.0\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("metadata.json"),
            r#"{"spec_version": "1.0", "app_name": "app", "hostname": "node",
                "kokkos_backend": "CUDA", "device_count": 4,
                "start_epoch_ns": 100, "mpi_rank": 3}"#,
        )
        .unwrap();

        let meta = load_trace_dir(dir.path()).unwrap().metadata.unwrap();
        assert_eq!(meta.device_count, Some(4));
        assert_eq!(meta.mpi_rank, Some(3));
    }
}
