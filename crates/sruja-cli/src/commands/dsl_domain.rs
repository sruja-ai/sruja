//! DSL domain commands

pub(crate) use super::sync_cmd;
pub(crate) use super::violation_shared;
pub(crate) use super::{parse_sruja_file, scan_repo_cached, CliError};

#[path = "check.rs"]
pub mod check;
#[path = "completions.rs"]
pub mod completions;
#[path = "dsl.rs"]
pub mod dsl;
#[path = "generate.rs"]
pub mod generate;
#[path = "watch.rs"]
pub mod watch;
