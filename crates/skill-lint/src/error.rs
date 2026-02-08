use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillLintError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Frontmatter parsing error: {0}")]
    FrontmatterParse(String),

    #[error("Broken link: {0}")]
    BrokenLink(String),

    #[error("Broken cross-reference: {0}")]
    BrokenCrossRef(String),

    #[error("Code compilation error: {0}")]
    CodeCompilation(String),

    #[error("Missing metadata in file: {0}")]
    MissingMetadata(String),

    #[error("Link check error: {0}")]
    LinkCheck(String),

    #[error("Cross-reference check error: {0}")]
    XrefCheck(String),

    #[error("Code test error: {0}")]
    CodeTest(String),

    #[error("Format check error: {0}")]
    FormatCheck(String),

    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Context analysis error: {0}")]
    Context(String),
}

pub type Result<T> = std::result::Result<T, SkillLintError>;
