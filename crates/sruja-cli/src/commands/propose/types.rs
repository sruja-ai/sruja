use super::CliError;

pub struct ProposeCreateRequest {
    pub description: String,
    pub workflow_id: Option<String>,
    pub add_elements: Vec<String>,
    pub add_relationships: Vec<String>,
    pub remove_elements: Vec<String>,
    pub remove_relationships: Vec<String>,
    pub format: String,
}

pub(super) struct AddElementSpec {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) label: String,
    pub(super) technology: Option<String>,
    pub(super) description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct ProposalLintSummary {
    pub(super) error_count: usize,
    pub(super) errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub(super) fn parse(s: &str) -> Result<Self, CliError> {
        match s.trim() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(CliError::validation(format!(
                "Invalid format '{}'. Expected 'text' or 'json'.",
                other
            ))),
        }
    }
}
