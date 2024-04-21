use core::str::FromStr;

use crate::errors::UnknownViolationCode;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Violation {
    ParseError,
    MismatchedDates,
    MismatchedStartTime,
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
            Violation::MismatchedStartTime => "starttime-mismatch",
            //TODO: Implement end time mismatch
            Violation::InvalidStartTime => "invalid-start-time",
            Violation::InvalidEndTime => "invalid-end-time",
            Violation::MissingDate => "missing-date",
            Violation::MissingStartTime => "missing-start-time",
            Violation::MissingEndTime => "missing-end-time",
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
            "starttime-mismatch" => Ok(Violation::MismatchedStartTime),
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

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;

    #[rstest]
    #[case("parse-error", true)]
    #[case("date-mismatch", true)]
    #[case("invalid-start-time", true)]
    #[case("invalid-end-time", true)]
    #[case("missing-date", true)]
    #[case("missing-start-time", true)]
    #[case("missing-end-time", true)]
    #[case("time-jumped", true)]
    #[case("negative-time-range", true)]
    #[case("incorrect-duration", true)]
    #[case("foobar", false)]
    fn string_conversion(#[case] s: &'static str, #[case] ok: bool) {
        let result = Violation::from_str(s);
        assert_eq!(result.is_ok(), ok);
        if ok {
            let violation = result.unwrap();
            assert_eq!(violation.as_str(), s);
            assert_eq!(format!("{violation}"), s);
        }
    }
}
