//! Runtime hotspot detection.
//!
//! Identifies span names by total duration and occurrence count for performance analysis.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::trace::ExecutionTrace;

/// A runtime hotspot: a span name with high duration or frequency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeHotspot {
    /// Span name
    pub span_name: String,
    /// Total duration across all occurrences (ms)
    pub total_duration_ms: i64,
    /// Number of occurrences
    pub count: usize,
}

/// Detects runtime hotspots from execution traces.
pub struct HotspotDetector {
    /// Minimum total duration (ms) to report
    pub min_duration_ms: i64,
    /// Maximum number of hotspots to return
    pub top_n: usize,
}

impl Default for HotspotDetector {
    fn default() -> Self {
        Self {
            min_duration_ms: 0,
            top_n: 20,
        }
    }
}

impl HotspotDetector {
    pub fn new() -> Self {
        Self::default()
    }

    fn aggregate(t: &ExecutionTrace, acc: &mut HashMap<String, (i64, usize)>) {
        let dur_ms = (t.end - t.start).num_milliseconds();
        acc.entry(t.name.clone())
            .and_modify(|(total, count)| {
                *total += dur_ms;
                *count += 1;
            })
            .or_insert((dur_ms, 1));

        for child in &t.children {
            Self::aggregate(child, acc);
        }
    }

    /// Detect hotspots from traces, sorted by total duration descending.
    pub fn detect(&self, traces: &[ExecutionTrace]) -> Vec<RuntimeHotspot> {
        let mut acc: HashMap<String, (i64, usize)> = HashMap::new();
        for t in traces {
            Self::aggregate(t, &mut acc);
        }

        let mut hotspots: Vec<RuntimeHotspot> = acc
            .into_iter()
            .filter(|(_, (dur, _))| *dur >= self.min_duration_ms)
            .map(|(span_name, (total_duration_ms, count))| RuntimeHotspot {
                span_name,
                total_duration_ms,
                count,
            })
            .collect();

        hotspots.sort_by(|a, b| b.total_duration_ms.cmp(&a.total_duration_ms));
        hotspots.truncate(self.top_n);
        hotspots
    }
}
