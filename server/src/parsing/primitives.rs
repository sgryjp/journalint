use core::ops::Range;
use core::result::Result;

use chumsky::prelude::*;

use crate::errors::JournalintError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub span: Range<usize>,
}

impl TryFrom<LooseDate> for chrono::NaiveDate {
    type Error = JournalintError;

    fn try_from(value: LooseDate) -> Result<Self, Self::Error> {
        let (year, month, day) = (value.year, value.month, value.day);
        match chrono::NaiveDate::from_ymd_opt(year, month, day) {
            Some(t) => Ok(t),
            None => Err(JournalintError::OutOfRangeDate { year, month, day }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTime {
    pub hour: u32,
    pub minute: u32,
    pub span: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTimeRange {
    pub start: LooseTime,
    pub end: LooseTime,
    pub span: Range<usize>,
}

impl TryFrom<LooseTime> for chrono::NaiveTime {
    type Error = JournalintError;

    fn try_from(value: LooseTime) -> Result<Self, Self::Error> {
        let (minute, h) = (value.minute % 60, value.minute - ((value.minute % 60) * 60));
        let hour = value.hour + h;
        match chrono::NaiveTime::from_hms_opt(hour, minute, 0) {
            Some(t) => Ok(t),
            None => Err(JournalintError::OutOfRangeTime { hour, minute }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duration {
    pub total_seconds: u32,
    pub span: Range<usize>,
}

fn _fixed_length_digits(len: usize) -> impl Parser<char, String, Error = Simple<char>> {
    one_of::<char, &str, Simple<char>>("0123456789")
        .repeated()
        .at_least(len)
        .at_most(len)
        .collect::<String>()
        .labelled("digits")
}

pub(super) fn date() -> impl Parser<char, LooseDate, Error = Simple<char>> {
    _fixed_length_digits(4)
        .then_ignore(just('-'))
        .then(_fixed_length_digits(2))
        .then_ignore(just('-'))
        .then(_fixed_length_digits(2))
        .map_with_span(|((y, m), d), span| LooseDate {
            year: str::parse::<i32>(&y).unwrap(),
            month: str::parse::<u32>(&m).unwrap(),
            day: str::parse::<u32>(&d).unwrap(),
            span,
        })
}

pub fn time() -> impl Parser<char, LooseTime, Error = Simple<char>> {
    _fixed_length_digits(2)
        .then_ignore(just(':'))
        .then(_fixed_length_digits(2))
        .map_with_span(|(h, s), span| LooseTime {
            hour: str::parse::<u32>(&h).unwrap(),
            minute: str::parse::<u32>(&s).unwrap(),
            span,
        })
}

pub fn timerange() -> impl Parser<char, LooseTimeRange, Error = Simple<char>> {
    time()
        .then_ignore(just('-'))
        .then(time())
        .map_with_span(|(s, e), span| LooseTimeRange {
            start: s,
            end: e,
            span,
        })
}

pub fn duration() -> impl Parser<char, Duration, Error = Simple<char>> {
    _fixed_length_digits(1)
        .then_ignore(just('.'))
        .then(_fixed_length_digits(2))
        .map(|(a, b)| format!("{}.{}", a, b))
        .try_map(|s, span| {
            str::parse::<f64>(&s).map_err(|e| Simple::custom(span, format!("{}", e)))
        })
        .map_with_span(|n, span| Duration {
            total_seconds: (n * 3600.0) as u32,
            span,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date() {
        let p = super::date();
        assert_eq!(
            p.parse("2006-01-02").unwrap(),
            LooseDate {
                year: 2006,
                month: 1,
                day: 2,
                span: 0..10
            },
        );
    }

    #[test]
    fn time() {
        let p = super::time();
        assert_eq!(
            p.parse("01:02").unwrap(),
            LooseTime {
                hour: 1,
                minute: 2,
                span: 0..5
            }
        );
        assert_eq!(
            p.parse("24:60").unwrap(),
            LooseTime {
                hour: 24,
                minute: 60,
                span: 0..5
            }
        );
        assert!(p.parse("24 :60").is_err());
        assert!(p.parse("24: 60").is_err());
    }

    #[test]
    fn timerange() {
        let p = super::timerange();
        assert_eq!(
            p.parse("01:02-03:04"),
            Ok(LooseTimeRange {
                start: LooseTime {
                    hour: 1,
                    minute: 2,
                    span: 0..5
                },
                end: LooseTime {
                    hour: 3,
                    minute: 4,
                    span: 6..11
                },
                span: Range { start: 0, end: 11 },
            })
        );
        assert!(p.parse("01 :02-03:04").is_err());
        assert!(p.parse("01: 02-03:04").is_err());
        assert!(p.parse("01:02 -03:04").is_err());
        assert!(p.parse("01:02- 03:04").is_err());
        assert!(p.parse("01:02-03 :04").is_err());
        assert!(p.parse("01:02-03: 04").is_err());
    }

    #[test]
    fn duration() {
        let p = super::duration();
        assert!(p.parse(".12").is_err());
        assert_eq!(
            p.parse("1.23").unwrap(),
            Duration {
                total_seconds: 4428,
                span: 0..4
            }
        );
        assert!(p.parse("12.34").is_err());
        assert!(p.parse("1.2").is_err());
        assert_eq!(
            p.parse("1.23").unwrap(),
            Duration {
                total_seconds: 4428,
                span: 0..4
            }
        );
        assert_eq!(
            p.parse("1.234").unwrap(),
            Duration {
                total_seconds: 4428,
                span: 0..4
            }
        );

        assert!(p.parse("1.2").is_err());
        assert!(p.parse(".123").is_err());
    }
}
