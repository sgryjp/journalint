use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("LSP communication error: {0}")]
    LspCommunicationError(String),

    #[error("I/O Error: {}", .source)]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {}", .source)]
    SerializationError {
        #[from]
        source: serde_json::error::Error,
    },
}
