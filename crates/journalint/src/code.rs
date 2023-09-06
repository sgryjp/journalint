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
}

impl From<&str> for Code {
    fn from(value: &str) -> Self {
        match value {
            "parse-error" => Code::ParseError,
            "date-mismatch" => Code::MismatchedDates,
            "invalid-start-time" => Code::InvalidStartTime,
            "invalid-end-time" => Code::InvalidEndTime,
            "missing-date" => Code::MissingDate,
            "missing-start-time" => Code::MissingStartTime,
            "missing-end-time" => Code::MissingEndTime,
            "time-jumped" => Code::TimeJumped,
            "negative-time-range" => Code::NegativeTimeRange,
            "incorrect-duration" => Code::IncorrectDuration,
            _ => panic!(),
        }
    }
}
