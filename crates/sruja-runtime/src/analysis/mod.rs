//! Runtime analysis: emergent cycles, hotspots, anomalies.

mod cycle;
mod hotspot;

pub use cycle::{CycleSeverity, EmergentCycle, EmergentCycleDetector};
pub use hotspot::{HotspotDetector, RuntimeHotspot};
