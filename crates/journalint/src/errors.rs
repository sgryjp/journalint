use crossbeam_channel::SendError;
use lsp_server::Message;
use lsp_server::ProtocolError;
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

impl From<SendError<Message>> for JournalintError {
    fn from(value: SendError<Message>) -> Self {
        JournalintError::LspCommunicationError(value.to_string())
    }
}

impl From<ProtocolError> for JournalintError {
    fn from(value: ProtocolError) -> Self {
        JournalintError::LspCommunicationError(value.to_string())
    }
}
