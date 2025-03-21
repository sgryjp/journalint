//! Provides AST related features.
//!
//! See module `parse` for parsing logic, and module `lint` for linting logic.
use std::ops::Range;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::errors::InvalidTimeValueError;

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

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_datetime(&self, date: NaiveDate) -> Result<DateTime<Utc>, InvalidTimeValueError> {
        match NaiveTime::parse_from_str(self.0.as_str(), "%H:%M") {
            Ok(t) => Ok(NaiveDateTime::new(date, t).and_utc()),
            Err(_) => {
                // Try parsing as it's beyond 24:00.
                let hhmm: Vec<&str> = self.0.split(':').collect();
                if hhmm.len() != 2 {
                    return Err(InvalidTimeValueError::new(
                        self.0.clone(),
                        "the time value is not in format \"HH:MM\"",
                    ));
                }
                let Ok(h) = str::parse::<u32>(hhmm[0]) else {
                    return Err(InvalidTimeValueError::new(
                        self.0.clone(),
                        "the hour is not a number",
                    ));
                };
                let Ok(m) = str::parse::<u32>(hhmm[1]) else {
                    return Err(InvalidTimeValueError::new(
                        self.0.clone(),
                        "the minute is not a number",
                    ));
                };
                if 60 < m {
                    return Err(InvalidTimeValueError::new(
                        self.0.clone(),
                        "minute value out of range",
                    ));
                }
                let num_days = h / 24;
                let time = NaiveTime::from_hms_opt(h - num_days * 24, m, 0)
                    .expect("failed to calculate time value");
                let Some(date) = date.checked_add_days(Days::new(u64::from(num_days))) else {
                    return Err(InvalidTimeValueError::new(
                        self.0.clone(),
                        format!("failed to calculate one date ahead of '{date}'"),
                    ));
                };
                Ok(NaiveDateTime::new(date, time).and_utc())
            }
        }
    }
}

pub trait Visitor<E> {
    #[warn(unused_results)]
    fn on_visit_fm_date(&mut self, _value: &NaiveDate, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_fm_start(&mut self, _value: &LooseTime, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_fm_end(&mut self, _value: &LooseTime, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_leave_fm(&mut self, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_entry(&mut self, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_start_time(&mut self, _value: &LooseTime, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_end_time(&mut self, _value: &LooseTime, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_duration(&mut self, _value: &Duration, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_code(&mut self, _value: &str, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_visit_activity(&mut self, _value: &str, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_leave_entry(&mut self, _span: &Range<usize>) -> Result<(), E> {
        Ok(())
    }

    #[warn(unused_results)]
    fn on_leave_journal(&mut self) -> Result<(), E> {
        Ok(())
    }
}

#[warn(unused_results)]
pub fn walk<E>(expr: &Expr, visitor: &mut impl Visitor<E>) -> Result<(), E> {
    match expr {
        Expr::FrontMatterDate { value, span } => visitor.on_visit_fm_date(value, span),
        Expr::FrontMatterStartTime { value, span } => visitor.on_visit_fm_start(value, span),
        Expr::FrontMatterEndTime { value, span } => visitor.on_visit_fm_end(value, span),
        Expr::FrontMatter {
            date,
            start,
            end,
            span,
        } => {
            walk(date, visitor)?;
            walk(start, visitor)?;
            walk(end, visitor)?;
            visitor.on_leave_fm(span)
        }
        Expr::StartTime { value, span } => visitor.on_visit_start_time(value, span),
        Expr::EndTime { value, span } => visitor.on_visit_end_time(value, span),
        Expr::Duration { value, span } => visitor.on_visit_duration(value, span),
        Expr::Code { value, span } => visitor.on_visit_code(value, span),
        Expr::Activity { value, span } => visitor.on_visit_activity(value, span),
        Expr::Entry {
            start,
            end,
            codes,
            duration,
            activity,
            span,
        } => {
            visitor.on_visit_entry(span)?;
            walk(start, visitor)?;
            walk(end, visitor)?;
            for code in codes {
                walk(code, visitor)?;
            }
            walk(duration, visitor)?;
            walk(activity, visitor)?;
            visitor.on_leave_entry(span)
        }
        Expr::Journal {
            front_matter,
            lines,
        } => {
            walk(front_matter, visitor)?;
            for line in lines {
                walk(line, visitor)?;
            }
            visitor.on_leave_journal()?;
            Ok(())
        }
        Expr::Error { reason: _, span: _ } => Ok(()),
        Expr::NonTargetLine => Ok(()),
    }
}
