use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;

use super::event::Event;
use super::sample::{PowerSample, PowerSeries};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub spec_version: String,
    pub app_name: Option<String>,
    pub hostname: Option<String>,
    pub kokkos_backend: Option<String>,
    pub start_epoch_ns: Option<u64>,
}

/// Unified trace data containing all events and telemetry samples.
#[derive(Debug, Clone)]
pub struct Trace {
    pub metadata: Option<Metadata>,
    pub events: Vec<Event>,
    /// One time-ordered series per (domain, device_id), in order of appearance.
    pub series: Vec<PowerSeries>,
}

impl Trace {
    pub fn new(
        metadata: Option<Metadata>,
        mut events: Vec<Event>,
        mut samples: Vec<PowerSample>,
    ) -> Self {
        // Parents come before their children when both start together
        events.sort_by_key(|e| (e.start_ns, Reverse(e.end_ns)));
        samples.sort_by_key(|s| s.timestamp_ns);

        let mut series: Vec<PowerSeries> = Vec::new();
        for sample in samples {
            let index = series
                .iter()
                .position(|s| s.domain == sample.domain && s.device_id == sample.device_id);
            match index {
                Some(i) => series[i].samples.push(sample),
                None => series.push(PowerSeries {
                    domain: sample.domain,
                    device_id: sample.device_id,
                    samples: vec![sample],
                }),
            }
        }

        Self {
            metadata,
            events,
            series,
        }
    }

    /// Index of the parent of each event, `None` for roots or unknown parents.
    pub fn event_parents(&self) -> Vec<Option<usize>> {
        let index: HashMap<u64, usize> = self
            .events
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id, i))
            .collect();

        self.events
            .iter()
            .enumerate()
            .map(|(i, e)| index.get(&e.parent_id).copied().filter(|&p| p != i))
            .collect()
    }

    /// Nesting depth of each event, 0 for roots.
    pub fn event_depths(&self) -> Vec<usize> {
        let parents = self.event_parents();
        let n = parents.len();
        let mut depths: Vec<Option<usize>> = vec![None; n];

        for i in 0..n {
            // Walk up to a root or an already resolved ancestor, bounded against cycles
            let mut chain = Vec::new();
            let mut current = Some(i);
            while let Some(c) = current {
                if depths[c].is_some() || chain.len() > n {
                    break;
                }
                chain.push(c);
                current = parents[c];
            }

            let mut depth = current.and_then(|c| depths[c]).map_or(0, |d| d + 1);
            for &c in chain.iter().rev() {
                depths[c] = Some(depth);
                depth += 1;
            }
        }

        depths.into_iter().map(|d| d.unwrap_or(0)).collect()
    }

    /// Earliest and latest timestamps over all events and power samples.
    pub fn time_bounds(&self) -> Option<(u64, u64)> {
        let min = self.events.iter().map(|e| e.start_ns);
        let max = self.events.iter().map(|e| e.end_ns);
        let min = min.chain(self.samples().map(|s| s.timestamp_ns)).min()?;
        let max = max.chain(self.samples().map(|s| s.timestamp_ns)).max()?;
        Some((min, max))
    }

    /// Iterate over every power sample of every series.
    pub fn samples(&self) -> impl Iterator<Item = &PowerSample> {
        self.series.iter().flat_map(|s| s.samples.iter())
    }
}
