//! Utility, federation, and setup commands

pub(crate) use super::discover;
pub(crate) use super::scan;
pub(crate) use super::{parse_sruja_file, CliError};

pub(crate) mod generate {
    pub use super::super::generate_prompt;
}

#[path = "classify.rs"]
pub mod classify;
#[path = "compliance.rs"]
pub mod compliance;
#[path = "error.rs"]
pub mod error;
#[path = "federation.rs"]
pub mod federation;
#[path = "init.rs"]
pub mod init;
#[path = "preflight.rs"]
pub mod preflight;
#[path = "run_export.rs"]
pub mod run_export;
#[path = "run_show.rs"]
pub mod run_show;
#[path = "sync_ide_rules.rs"]
pub mod sync_ide_rules;
#[path = "version.rs"]
pub mod version;
#[path = "violation_shared.rs"]
pub mod violation_shared;
#[path = "repo_manifest.rs"]
pub mod repo_manifest;

pub use classify::{classify, ClassifyOptions};
pub use sync_ide_rules::{sync_ide_rules, SyncIdeRulesOptions};
