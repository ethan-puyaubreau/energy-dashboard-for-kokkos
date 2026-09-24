//! In-memory representation of a trace: events, power samples and metadata.

pub mod event;
pub mod sample;
pub mod trace;

pub use event::{Event, RegionCategory};
pub use sample::{DeviceDomain, PowerSample, PowerSeries};
pub use trace::{Metadata, Trace};
