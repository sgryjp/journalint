//! Provides AST related features.
//!
//! See module `parse` for parsing logic, and module `lint` for linting logic.
use std::ops::Range;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::errors::JournalintError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    FrontMatterDate {
        value: NaiveDate,
        span: Range<usize>,
    },
    FrontMatterStartTime {
        value: LooseTime,
        span: Range<usize>,
    },
    FrontMatterEndTime {
        value: LooseTime,
        span: Range<usize>,
    },
    FrontMatter {
        date: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
        span: Range<usize>,
    },

    StartTime {
        value: LooseTime,
        span: Range<usize>,
    },
    EndTime {
        value: LooseTime,
        span: Range<usize>,
    },
    Duration {
        value: Duration,
        span: Range<usize>,
    },
    Code {
        value: String,
        span: Range<usize>,
    },
    Activity {
        value: String,
        span: Range<usize>,
    },
    Entry {
        start: Box<Expr>,
        end: Box<Expr>,
        codes: Vec<Expr>,
        duration: Box<Expr>,
        activity: Box<Expr>,
        span: Range<usize>,
    },
    Journal {
        front_matter: Box<Expr>,
        lines: Vec<Expr>,
    },

    Error {
        reason: String,
        span: Range<usize>,
    },
    NonTargetLine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTime(String);

impl LooseTime {
    pub fn new<T: Into<String>>(value: T) -> LooseTime {
        LooseTime(value.into())
    }

    pub fn to_datetime(&self, date: NaiveDate) -> Result<DateTime<Utc>, JournalintError> {
        match NaiveTime::parse_from_str(self.0.as_str(), "%H:%M") {
            Ok(t) => {
                let datetime = NaiveDateTime::new(date, t);
                Ok(DateTime::from_utc(datetime, Utc))
            }
            Err(e) => {
                // Try parsing as it's beyond 24:00.
                let hhmm: Vec<&str> = self.0.split(':').collect();
                if hhmm.len() != 2 {
                    return Err(JournalintError::ParseError(format!(
                        "the time value is not in format \"HH:MM\": '{}'",
                        self.0
                    )));
                }
                let Ok(h) = str::parse::<u32>(hhmm[0]) else {
                    return Err(JournalintError::ParseError(format!(
                        "the hour is not a number: '{}'",
                        self.0
                    )));
                };
                let Ok(m) = str::parse::<u32>(hhmm[1]) else {
                    return Err(JournalintError::ParseError(format!(
                        "the minute is not a number: '{}'",
                        self.0
                    )));
                };
                if 60 < m {
                    return Err(JournalintError::ParseError(format!(
                        "invalid minute value: {}: '{}'",
                        e, self.0
                    )));
                }
                let num_days = h / 24;
                let time = NaiveTime::from_hms_opt(h - num_days * 24, m, 0).unwrap();
                let Some(date) = date.checked_add_days(Days::new(u64::from(num_days))) else {
                    return Err(JournalintError::ParseError(format!(
                        "failed to calculate one date ahead of '{date}'"
                    )));
                };
                let datetime = NaiveDateTime::new(date, time);
                Ok(DateTime::from_utc(datetime, Utc))
            }
        }
    }
}
