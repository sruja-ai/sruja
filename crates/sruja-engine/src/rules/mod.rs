//! Validation rules for Sruja architectures

pub mod unique_id;
pub mod cycle;
pub mod orphan;
pub mod valid_ref;
pub mod simplicity;
pub mod layer_violation;
pub mod scenario_validation;
pub mod database_isolation;
pub mod public_interface_documentation;
pub mod slo_validation;
pub mod properties_validation;

pub use unique_id::UniqueIdRule;
pub use cycle::CycleDetectionRule;
pub use orphan::OrphanDetectionRule;
pub use valid_ref::ValidRefRule;
pub use simplicity::SimplicityRule;
pub use layer_violation::LayerViolationRule;
pub use scenario_validation::ScenarioValidationRule;
pub use database_isolation::DatabaseIsolationRule;
pub use public_interface_documentation::PublicInterfaceDocumentationRule;
pub use slo_validation::SloValidationRule;
pub use properties_validation::PropertiesValidationRule;
