use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryStoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("agent memory: {0}")]
    Agent(#[from] sruja_agent::memory::MemoryError),
    #[error("{0}")]
    Validation(String),
}
