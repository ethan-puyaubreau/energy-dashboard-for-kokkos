//! Hardware power samples and per-device series.

use serde::{Deserialize, Serialize};

/// Hardware domain a power sample was measured on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceDomain {
    /// Graphics processing unit.
    #[serde(rename = "GPU")]
    Gpu,
    /// CPU package of a socket.
    #[serde(rename = "CPU_PKG")]
    CpuPkg,
    /// DRAM attached to a socket.
    #[serde(rename = "CPU_DRAM")]
    CpuDram,
    /// Whole compute node.
    #[serde(rename = "NODE")]
    Node,
    /// Any domain unknown to this version.
    #[serde(other)]
    Other,
}

impl std::fmt::Display for DeviceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gpu => write!(f, "GPU"),
            Self::CpuPkg => write!(f, "CPU_PKG"),
            Self::CpuDram => write!(f, "CPU_DRAM"),
            Self::Node => write!(f, "NODE"),
            Self::Other => write!(f, "OTHER"),
        }
    }
}

/// Instantaneous hardware power sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSample {
    /// Measurement timestamp in nanoseconds since UNIX epoch.
    pub timestamp_ns: u64,
    /// Measured hardware domain.
    pub domain: DeviceDomain,
    /// Device index within the domain.
    pub device_id: u32,
    /// Instantaneous power in Watts.
    pub power_watts: f64,
    /// Hardware cumulative energy counter in Joules.
    ///
    /// Parsed for format compatibility but ignored by the analysis.
    #[serde(default)]
    pub energy_joules: Option<f64>,
}

/// Time-ordered power samples of a single device.
#[derive(Debug, Clone)]
pub struct PowerSeries {
    /// Measured hardware domain.
    pub domain: DeviceDomain,
    /// Device index within the domain.
    pub device_id: u32,
    /// Samples sorted by timestamp.
    pub samples: Vec<PowerSample>,
}
