use crossbeam_channel::SendError;
use journalint_parse::errors::UnknownViolationCode;
use lsp_server::Message;
use lsp_server::ProtocolError;
use lsp_types::Url;
use thiserror::Error;

use journalint_parse::errors::InvalidTimeValueError;

// ----------------------------------------------------------------------------

/// Error for CLI main function.
#[derive(Default)]
pub struct CliError {
    exit_status: i32,
    message: Option<String>,
}

impl CliError {
    pub fn new(exit_status: i32) -> Self {
        Self {
            exit_status,
            message: None,
        }
    }

    pub fn with_message(&self, message: String) -> Self {
        Self {
            message: Some(message),
            ..*self
        }
    }

    pub fn exit_status(&self) -> i32 {
        self.exit_status
    }

    pub fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }
}

// ----------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("UNEXPECTED ERROR: {0}")]
    UnexpectedError(String),

    #[error("{}", source)]
    UnknownViolationCode {
        #[from]
        source: UnknownViolationCode,
    },

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Unexpected arguments: {0}")]
    UnexpectedArguments(String),

    #[error("Unsupported URL: {url}")]
    UnsupportedUrl { url: Url },

    #[error("Doucument not found: {url}")]
    DocumentNotFound { url: Url },

    #[error("Parse error: {}", .source)]
    InvalidTimeValueError {
        #[from]
        source: InvalidTimeValueError,
    },

    #[error("Required value is missing: {name}")]
    MissingRequiredValue { name: String },

    #[error("Target not found for command '{command}'")]
    CommandTargetNotFound { command: String },

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

    #[error("Parsing date or time failed: {}", .source)]
    ChronoParseError {
        #[from]
        source: chrono::ParseError,
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
