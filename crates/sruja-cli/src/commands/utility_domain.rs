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
#[path = "generate_skill.rs"]
pub mod generate_skill;
#[path = "init.rs"]
pub mod init;
#[path = "preflight.rs"]
pub mod preflight;
#[path = "repo_manifest.rs"]
pub mod repo_manifest;
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

pub use classify::{classify, ClassifyOptions};
pub use generate_skill::{generate_skill_prompt, GenerateSkillPromptOptions};
pub use sync_ide_rules::{sync_ide_rules, SyncIdeRulesOptions};
