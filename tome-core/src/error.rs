use thiserror::Error;

pub type Result<T> = std::result::Result<T, TomeError>;

#[derive(Debug, Error)]
pub enum TomeError {
    #[error("TLD not found: {0}")]
    TldNotFound(String),

    #[error("Record type not found: {0}")]
    RecordTypeNotFound(String),

    #[error("Glossary term not found: {0}")]
    GlossaryTermNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl TomeError {
    /// Returns a sanitized error message safe for external API exposure.
    pub fn sanitized_message(&self) -> String {
        match self {
            TomeError::TldNotFound(tld) => format!("TLD not found: {tld}"),
            TomeError::RecordTypeNotFound(rt) => format!("Record type not found: {rt}"),
            TomeError::GlossaryTermNotFound(term) => format!("Glossary term not found: {term}"),
            TomeError::InvalidQuery(msg) => format!("Invalid query: {msg}"),
            TomeError::DataError(_) => "Internal data error".to_string(),
            TomeError::SerializationError(_) => "Serialization error".to_string(),
        }
    }
}
