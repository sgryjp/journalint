use core::ops::Range;
use core::result::Result;

use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};
use chumsky::prelude::*;

use crate::errors::JournalintError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseDate {
    value: NaiveDate,
    span: Range<usize>,
}

impl LooseDate {
    pub fn new(value: NaiveDate, span: Range<usize>) -> Self {
        LooseDate { value, span }
    }

    pub fn value(&self) -> NaiveDate {
        self.value
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTime {
    hour: u32,
    minute: u32,
    span: Range<usize>,
}

impl LooseTime {
    pub fn new(hour: u32, minute: u32, span: Range<usize>) -> Self {
        LooseTime { hour, minute, span }
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn minute(&self) -> u32 {
        self.minute
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn into_datetime(&self, date: &LooseDate) -> Option<NaiveDateTime> {
        let day = self.hour / 24;
        let hour = self.hour - day * 24;
        let min = self.minute;
        dbg!(day, hour, min);
        NaiveDateTime::new(date.value, NaiveTime::from_hms_opt(hour, min, 0).unwrap())
            .checked_add_days(Days::new(day as u64))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTimeRange {
    start: LooseTime,
    end: LooseTime,
    span: Range<usize>,
}

impl LooseTimeRange {
    pub fn new(start: LooseTime, end: LooseTime, span: Range<usize>) -> Self {
        Self { start, end, span }
    }

    pub fn end(&self) -> &LooseTime {
        &self.end
    }

    pub fn start(&self) -> &LooseTime {
        &self.start
    }
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
    value: std::time::Duration,
    span: Range<usize>,
}

impl Duration {
    pub fn new(value: std::time::Duration, span: Range<usize>) -> Self {
        Duration { value, span }
    }

    pub fn value(&self) -> &std::time::Duration {
        &self.value
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
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
        .try_map(|((y, m), d), span| {
            NaiveDate::from_ymd_opt(
                str::parse::<i32>(&y).unwrap(),
                str::parse::<u32>(&m).unwrap(),
                str::parse::<u32>(&d).unwrap(),
            )
            .map(|d| LooseDate::new(d, span.clone()))
            .ok_or(Simple::custom(
                span,
                format!("invalid date: {:4}-{}-{}", y, m, d),
            ))
        })
}

pub fn time() -> impl Parser<char, LooseTime, Error = Simple<char>> {
    _fixed_length_digits(2)
        .then_ignore(just(':'))
        .then(_fixed_length_digits(2))
        .map_with_span(|(h, s), span| {
            LooseTime::new(
                str::parse::<u32>(&h).unwrap(),
                str::parse::<u32>(&s).unwrap(),
                span,
            )
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
        .map_with_span(|n, span| Duration::new((n * 3600.0) as u32, span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date() {
        let p = super::date();
        assert_eq!(
            p.parse("2006-01-02").unwrap(),
            LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 0..10),
        );

        assert!(p.parse("2006-01-00").is_err());
    }

    #[test]
    fn time() {
        let p = super::time();
        assert_eq!(p.parse("01:02").unwrap(), LooseTime::new(1, 2, 0..5));
        assert_eq!(p.parse("24:60").unwrap(), LooseTime::new(24, 60, 0..5));
        assert!(p.parse("24 :60").is_err());
        assert!(p.parse("24: 60").is_err());
    }

    #[test]
    fn timerange() {
        let p = super::timerange();
        assert_eq!(
            p.parse("01:02-03:04"),
            Ok(LooseTimeRange {
                start: LooseTime::new(1, 2, 0..5),
                end: LooseTime::new(3, 4, 6..11),
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
