use core::str::FromStr;

use crate::commands::{AutofixCommand, Command};
use crate::errors::JournalintError;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Code {
    ParseError,
    MismatchedDates,
    InvalidStartTime,
    InvalidEndTime,
    MissingDate,
    MissingStartTime,
    MissingEndTime,
    TimeJumped,
    NegativeTimeRange,
    IncorrectDuration,
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Code {
    pub fn as_str(&self) -> &str {
        match self {
            Code::ParseError => "parse-error",
            Code::MismatchedDates => "date-mismatch",
            Code::InvalidStartTime => "invalid-start-time",
            Code::InvalidEndTime => "invalid-end-time",
            Code::MissingDate => "missing-date",
            Code::MissingStartTime => "missing-start-time",
            Code::MissingEndTime => "missing-end-time",
            //TODO: Implement start/end time mismatch
            Code::TimeJumped => "time-jumped",
            Code::NegativeTimeRange => "negative-time-range",
            Code::IncorrectDuration => "incorrect-duration",
        }
    }

    /// Get default auto-fix command for the diagnostic code.
    pub fn default_autofix(&self) -> Option<impl Command> {
        match self {
            Code::ParseError => None,
            Code::MismatchedDates => Some(AutofixCommand::UseDateInFilename),
            Code::InvalidStartTime => None,
            Code::InvalidEndTime => None,
            Code::MissingDate => None,
            Code::MissingStartTime => None,
            Code::MissingEndTime => None,
            Code::TimeJumped => Some(AutofixCommand::ReplaceWithPreviousEndTime),
            Code::NegativeTimeRange => None,
            Code::IncorrectDuration => Some(AutofixCommand::RecalculateDuration),
        }
    }
}

impl FromStr for Code {
    type Err = JournalintError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse-error" => Ok(Code::ParseError),
            "date-mismatch" => Ok(Code::MismatchedDates),
            "invalid-start-time" => Ok(Code::InvalidStartTime),
            "invalid-end-time" => Ok(Code::InvalidEndTime),
            "missing-date" => Ok(Code::MissingDate),
            "missing-start-time" => Ok(Code::MissingStartTime),
            "missing-end-time" => Ok(Code::MissingEndTime),
            "time-jumped" => Ok(Code::TimeJumped),
            "negative-time-range" => Ok(Code::NegativeTimeRange),
            "incorrect-duration" => Ok(Code::IncorrectDuration),
            _ => Err(JournalintError::UnknownCode(s.to_string())),
        }
    }
}
