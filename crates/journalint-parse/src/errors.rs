use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalintParseError {
    #[error("Parse error: {0}")]
    ParseError(String),
}
