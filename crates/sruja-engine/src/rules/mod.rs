//! Validation rules for Sruja architectures

pub mod unique_id;
pub mod cycle;
pub mod orphan;
pub mod valid_ref;
pub mod simplicity;
pub mod layer_violation;
pub mod scenario_validation;

pub use unique_id::UniqueIdRule;
pub use cycle::CycleDetectionRule;
pub use orphan::OrphanDetectionRule;
pub use valid_ref::ValidRefRule;
pub use simplicity::SimplicityRule;
pub use layer_violation::LayerViolationRule;
pub use scenario_validation::ScenarioValidationRule;
