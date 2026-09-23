use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceDomain {
    #[serde(rename = "GPU")]
    Gpu,
    #[serde(rename = "CPU_PKG")]
    CpuPkg,
    #[serde(rename = "CPU_DRAM")]
    CpuDram,
    #[serde(rename = "NODE")]
    Node,
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
    pub timestamp_ns: u64,
    pub domain: DeviceDomain,
    pub device_id: u32,
    pub power_watts: f64,
    #[serde(default)]
    pub energy_joules: Option<f64>,
}
