//! Kokkos execution events.

use serde::{Deserialize, Serialize};

/// Kokkos construct an event was recorded for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionCategory {
    /// Region opened with `Kokkos::Profiling::pushRegion`.
    #[serde(rename = "USER_REGION")]
    UserRegion,
    /// `Kokkos::parallel_for` kernel.
    #[serde(rename = "PARALLEL_FOR")]
    ParallelFor,
    /// `Kokkos::parallel_reduce` kernel.
    #[serde(rename = "PARALLEL_REDUCE")]
    ParallelReduce,
    /// `Kokkos::parallel_scan` kernel.
    #[serde(rename = "PARALLEL_SCAN")]
    ParallelScan,
    /// `Kokkos::deep_copy` memory transfer.
    #[serde(rename = "DEEP_COPY")]
    DeepCopy,
    /// Any category unknown to this version.
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
    /// Unique identifier within the run.
    pub id: u64,
    /// Identifier of the enclosing event, 0 for a root event.
    pub parent_id: u64,
    /// Region or kernel label.
    pub name: String,
    /// Kokkos category of the event.
    pub category: RegionCategory,
    /// Start timestamp in nanoseconds since UNIX epoch.
    pub start_ns: u64,
    /// End timestamp in nanoseconds since UNIX epoch.
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
