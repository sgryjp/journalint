use core::str::FromStr;

use crate::errors::UnknownRule;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Rule {
    ParseError,
    MismatchedDates,
    MismatchedStartTime,
    MismatchedEndTime,
    InvalidStartTime,
    InvalidEndTime,
    MissingDate,
    MissingStartTime,
    MissingEndTime,
    TimeJumped,
    NegativeTimeRange,
    IncorrectDuration,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Rule {
    pub fn as_str(&self) -> &str {
        match self {
            Rule::ParseError => "parse-error",
            Rule::MismatchedDates => "date-mismatch",
            Rule::MismatchedStartTime => "starttime-mismatch",
            Rule::MismatchedEndTime => "endtime-mismatch",
            Rule::InvalidStartTime => "invalid-start-time",
            Rule::InvalidEndTime => "invalid-end-time",
            Rule::MissingDate => "missing-date",
            Rule::MissingStartTime => "missing-start-time",
            Rule::MissingEndTime => "missing-end-time",
            Rule::TimeJumped => "time-jumped",
            Rule::NegativeTimeRange => "negative-time-range",
            Rule::IncorrectDuration => "incorrect-duration",
        }
    }
}

impl FromStr for Rule {
    type Err = UnknownRule;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse-error" => Ok(Rule::ParseError),
            "date-mismatch" => Ok(Rule::MismatchedDates),
            "starttime-mismatch" => Ok(Rule::MismatchedStartTime),
            "invalid-start-time" => Ok(Rule::InvalidStartTime),
            "invalid-end-time" => Ok(Rule::InvalidEndTime),
            "missing-date" => Ok(Rule::MissingDate),
            "missing-start-time" => Ok(Rule::MissingStartTime),
            "missing-end-time" => Ok(Rule::MissingEndTime),
            "time-jumped" => Ok(Rule::TimeJumped),
            "negative-time-range" => Ok(Rule::NegativeTimeRange),
            "incorrect-duration" => Ok(Rule::IncorrectDuration),
            _ => Err(UnknownRule {
                rule: s.to_string(),
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
        let result = Rule::from_str(s);
        assert_eq!(result.is_ok(), ok);
        if ok {
            let rule = result.unwrap();
            assert_eq!(rule.as_str(), s);
            assert_eq!(format!("{rule}"), s);
        }
    }
}
