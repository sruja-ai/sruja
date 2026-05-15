//! Intent and AI domain commands

pub(crate) use super::context;
pub(crate) use super::scan;
pub(crate) use super::{parse_sruja_file, scan_repo_cached, CliError};

#[path = "agent.rs"]
pub mod agent;
#[path = "agent_plan.rs"]
pub mod agent_plan;
#[path = "agent_run.rs"]
pub mod agent_run;
#[path = "ai.rs"]
pub mod ai;
#[path = "critique.rs"]
pub mod critique;
#[path = "evolution.rs"]
pub mod evolution;
#[path = "focus.rs"]
pub mod focus;
#[path = "ingest.rs"]
pub mod ingest;
#[path = "intent.rs"]
pub mod intent;
#[path = "onboard.rs"]
pub mod onboard;
#[path = "propose.rs"]
pub mod propose;
#[path = "remediation.rs"]
pub mod remediation;
