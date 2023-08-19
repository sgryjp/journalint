use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),

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
