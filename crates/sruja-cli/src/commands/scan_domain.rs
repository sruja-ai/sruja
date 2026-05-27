//! Scan and Graph domain commands

pub(crate) use super::author;
pub(crate) use super::critique;
pub(crate) use super::violation_shared;
pub(crate) use super::{parse_sruja_file, scan_repo_cached, scan_repo_cached_with_opts, CliError};

#[path = "context_graph.rs"]
pub mod context_graph;
#[path = "context_score.rs"]
pub mod context_score;
#[path = "discover.rs"]
pub mod discover;
#[path = "explore.rs"]
pub mod explore;
#[path = "health.rs"]
pub mod health;
#[path = "impact.rs"]
pub mod impact;
#[path = "index.rs"]
pub mod index;
#[path = "mcp/mod.rs"]
pub mod mcp;
#[path = "review.rs"]
pub mod review;
#[path = "scan/mod.rs"]
pub mod scan;
#[path = "status.rs"]
pub mod status;
#[path = "sync_cmd.rs"]
pub mod sync_cmd;
#[path = "why.rs"]
pub mod why;
