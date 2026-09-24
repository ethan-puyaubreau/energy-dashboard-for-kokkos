use serde::{Deserialize, Serialize};

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
        events.sort_by_key(|e| e.start_ns);
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

    /// Iterate over every power sample of every series.
    pub fn samples(&self) -> impl Iterator<Item = &PowerSample> {
        self.series.iter().flat_map(|s| s.samples.iter())
    }
}
