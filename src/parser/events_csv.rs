use crate::model::Event;
use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

/// Parse events.csv into a vector of Event structs.
pub fn parse_events_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Event>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open events file: {}", path.as_ref().display()))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut events = Vec::new();
    for (index, result) in rdr.deserialize().enumerate() {
        let event: Event = result.with_context(|| {
            format!(
                "Malformed record in events file: {}",
                path.as_ref().display()
            )
        })?;

        if event.end_ns < event.start_ns {
            anyhow::bail!(
                "Event ends before it starts in events file {} at record {}",
                path.as_ref().display(),
                index + 1
            );
        }
        events.push(event);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_events() {
        let data = "id,parent_id,name,category,start_ns,end_ns\n1,0,Main,USER_REGION,100,500\n2,1,Kernel,PARALLEL_FOR,150,450\n";
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let events: Vec<Event> = rdr.deserialize().map(|r| r.unwrap()).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name, "Main");
        assert_eq!(events[0].duration_ns(), 400);
        assert_eq!(
            events[1].category,
            crate::model::RegionCategory::ParallelFor
        );
    }

    #[test]
    fn test_reject_event_ending_before_start() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut file,
            b"id,parent_id,name,category,start_ns,end_ns\n1,0,Main,USER_REGION,500,100\n",
        )
        .unwrap();
        assert!(parse_events_csv(file.path()).is_err());
    }
}
