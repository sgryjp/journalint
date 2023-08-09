use lsp_types::Position;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Parse error")]
    FatalParseError {
        position: Option<Position>,
        filename: Option<String>,
        message: String,
    },

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

impl From<serde_yaml::Error> for JournalintError {
    fn from(value: serde_yaml::Error) -> Self {
        let position = value.location().map(|l| Position {
            line: l.line() as u32,
            character: l.column() as u32,
        });

        JournalintError::FatalParseError {
            position,
            filename: None, // TODO: Not a good implementation
            message: value.to_string(),
        }
    }
}
