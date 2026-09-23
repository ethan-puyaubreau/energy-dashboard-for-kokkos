use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionCategory {
    #[serde(rename = "USER_REGION")]
    UserRegion,
    #[serde(rename = "PARALLEL_FOR")]
    ParallelFor,
    #[serde(rename = "PARALLEL_REDUCE")]
    ParallelReduce,
    #[serde(rename = "PARALLEL_SCAN")]
    ParallelScan,
    #[serde(rename = "DEEP_COPY")]
    DeepCopy,
    #[serde(other)]
    Other,
}

impl std::fmt::Display for RegionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserRegion => write!(f, "USER_REGION"),
            Self::ParallelFor => write!(f, "PARALLEL_FOR"),
            Self::ParallelReduce => write!(f, "PARALLEL_REDUCE"),
            Self::ParallelScan => write!(f, "PARALLEL_SCAN"),
            Self::DeepCopy => write!(f, "DEEP_COPY"),
            Self::Other => write!(f, "OTHER"),
        }
    }
}

/// Execution event representing a Kokkos region, kernel or data movement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    pub parent_id: u64,
    pub name: String,
    pub category: RegionCategory,
    pub start_ns: u64,
    pub end_ns: u64,
}

impl Event {
    /// Duration of the event in nanoseconds.
    #[inline]
    pub fn duration_ns(&self) -> u64 {
        self.end_ns.saturating_sub(self.start_ns)
    }

    /// Duration of the event in seconds.
    #[inline]
    pub fn duration_sec(&self) -> f64 {
        self.duration_ns() as f64 / 1_000_000_000.0
    }
}
