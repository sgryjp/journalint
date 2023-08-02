use lsp_types::Position;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Out of range date: year={year}, month={month}, day={day}")]
    OutOfRangeDate { year: i32, month: u32, day: u32 },

    #[error("Out of range time: hour={hour}, minute={minute}")]
    OutOfRangeTime { hour: u32, minute: u32 },

    #[error("Parse error")]
    FatalParseError {
        position: Option<Position>,
        filename: Option<String>,
        message: String,
    },

    #[error("Invalid argument: {0}")]
    ArgumentError(String),

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

impl JournalintError {
    pub fn from_yaml_error(filename: Option<String>, err: serde_yaml::Error) -> Self {
        let position = err.location().map(|l| Position {
            line: l.line() as u32,
            character: l.column() as u32,
        });

        JournalintError::FatalParseError {
            position,
            filename,
            message: err.to_string(),
        }
    }
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
