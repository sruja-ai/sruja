//! Domain clustering and bounded context detection.

mod dbscan;
mod domain;
mod context;

pub use dbscan::{dbscan, ClusterLabels};
pub use domain::{DomainCluster, DomainClusterer};
pub use context::{BoundedContext, BoundedContextDetector};
