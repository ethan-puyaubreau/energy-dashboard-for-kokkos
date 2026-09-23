use serde::{Deserialize, Serialize};

use super::event::Event;
use super::sample::PowerSample;

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
    pub samples: Vec<PowerSample>,
}

impl Trace {
    pub fn new(metadata: Option<Metadata>, mut events: Vec<Event>, mut samples: Vec<PowerSample>) -> Self {
        events.sort_by_key(|e| e.start_ns);
        samples.sort_by_key(|s| s.timestamp_ns);
        Self {
            metadata,
            events,
            samples,
        }
    }
}
