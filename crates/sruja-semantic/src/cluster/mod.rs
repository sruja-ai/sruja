//! Domain clustering and bounded context detection.

mod context;
mod dbscan;
mod domain;

pub use context::{BoundedContext, BoundedContextDetector};
pub use dbscan::{dbscan, ClusterLabels};
pub use domain::{DomainCluster, DomainClusterer};
