//! Validation rules for Sruja architectures

pub mod unique_id;
pub mod cycle;
pub mod orphan;
pub mod valid_ref;

pub use unique_id::UniqueIdRule;
pub use cycle::CycleDetectionRule;
pub use orphan::OrphanDetectionRule;
pub use valid_ref::ValidRefRule;
