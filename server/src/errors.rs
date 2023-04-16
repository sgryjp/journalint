use std::path::PathBuf;

use lsp_types::Position;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintError {
    #[error("Parse error")]
    FatalParseError {
        pos: Option<Position>,
        path: PathBuf,
        msg: String,
    },

    #[error("Invalid argument: {0}")]
    ArgumentError(String),

    #[error("I/O Error: {}", .source)]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Command line argument error: {}", .source)]
    ClapError {
        #[from]
        source: clap::error::Error,
    },

    #[error("Serialization error: {}", .source)]
    SerializationError {
        #[from]
        source: serde_json::error::Error,
    },

    #[error("Protocol error: {}", .source)]
    ProtocolError {
        #[from]
        source: lsp_server::ProtocolError,
    },
}
