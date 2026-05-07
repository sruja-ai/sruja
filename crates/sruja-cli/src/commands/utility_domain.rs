//! Utility, federation, and setup commands

pub(crate) use super::discover;
pub(crate) use super::scan;
pub(crate) use super::{parse_sruja_file, CliError};

pub(crate) mod generate {
    pub use super::super::generate_prompt;
}

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
#[path = "version.rs"]
pub mod version;
#[path = "violation_shared.rs"]
pub mod violation_shared;
