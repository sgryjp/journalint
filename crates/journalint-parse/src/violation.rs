use core::str::FromStr;

use crate::errors::UnknownViolationCode;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Violation {
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

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Violation {
    pub fn as_str(&self) -> &str {
        match self {
            Violation::ParseError => "parse-error",
            Violation::MismatchedDates => "date-mismatch",
            Violation::InvalidStartTime => "invalid-start-time",
            Violation::InvalidEndTime => "invalid-end-time",
            Violation::MissingDate => "missing-date",
            Violation::MissingStartTime => "missing-start-time",
            Violation::MissingEndTime => "missing-end-time",
            //TODO: Implement start/end time mismatch
            Violation::TimeJumped => "time-jumped",
            Violation::NegativeTimeRange => "negative-time-range",
            Violation::IncorrectDuration => "incorrect-duration",
        }
    }
}

impl FromStr for Violation {
    type Err = UnknownViolationCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse-error" => Ok(Violation::ParseError),
            "date-mismatch" => Ok(Violation::MismatchedDates),
            "invalid-start-time" => Ok(Violation::InvalidStartTime),
            "invalid-end-time" => Ok(Violation::InvalidEndTime),
            "missing-date" => Ok(Violation::MissingDate),
            "missing-start-time" => Ok(Violation::MissingStartTime),
            "missing-end-time" => Ok(Violation::MissingEndTime),
            "time-jumped" => Ok(Violation::TimeJumped),
            "negative-time-range" => Ok(Violation::NegativeTimeRange),
            "incorrect-duration" => Ok(Violation::IncorrectDuration),
            _ => Err(UnknownViolationCode {
                code: s.to_string(),
            }),
        }
    }
}
