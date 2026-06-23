//! Indexed cross-session memory (SQLite + FTS5). Native only — not built for WASM.
//!
//! Sources: agent learnings (hypothesis), context events (mostly hypothesis), decision
//! records under `.sruja/decisions/` (reviewed truth). Never writes to `repo.sruja`.

mod error;
pub mod memory_backend;
mod store;

pub use error::MemoryStoreError;
pub use memory_backend::IndexedMemory;
pub use store::{
    MemorySearchHit, MemoryStore, MemoryTimelineEntry, MemoryTimelineResult, SearchMemoryOptions,
    TimelineOptions,
};
