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
                let time = NaiveTime::from_hms_opt(h - num_days * 24, m, 0)
                    .expect("failed to calculate time value");
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

pub trait Visitor {
    fn on_visit_fm_date(&mut self, value: &NaiveDate, span: &Range<usize>);
    fn on_visit_fm_start(&mut self, value: &LooseTime, span: &Range<usize>);
    fn on_visit_fm_end(&mut self, value: &LooseTime, span: &Range<usize>);
    fn on_leave_fm(&mut self, date: &Expr, start: &Expr, end: &Expr, span: &Range<usize>);
    fn on_visit_entry(
        // TODO: Remove child expressions from parameters
        &mut self,
        start_time: &Expr,
        end_time: &Expr,
        codes: &[Expr],
        duration: &Expr,
        activity: &Expr,
        span: &Range<usize>,
    );
    fn on_visit_start_time(&mut self, value: &LooseTime, span: &Range<usize>);
    fn on_visit_end_time(&mut self, value: &LooseTime, span: &Range<usize>);
    fn on_visit_duration(&mut self, value: &Duration, span: &Range<usize>);
    fn on_visit_code(&mut self, value: &str, span: &Range<usize>);
    fn on_visit_activity(&mut self, value: &str, span: &Range<usize>);
    fn on_leave_entry(
        &mut self,
        start_time: &Expr,
        end_time: &Expr,
        codes: &[Expr],
        duration: &Expr,
        activity: &Expr,
        span: &Range<usize>,
    );
}

pub fn walk(expr: &Expr, visitor: &mut impl Visitor) {
    match expr {
        Expr::FrontMatterDate { value, span } => {
            visitor.on_visit_fm_date(value, span);
        }
        Expr::FrontMatterStartTime { value, span } => {
            visitor.on_visit_fm_start(value, span);
        }
        Expr::FrontMatterEndTime { value, span } => {
            visitor.on_visit_fm_end(value, span);
        }
        Expr::FrontMatter {
            date,
            start,
            end,
            span,
        } => {
            walk(date, visitor);
            walk(start, visitor);
            walk(end, visitor);
            visitor.on_leave_fm(date, start, end, span);
        }
        Expr::StartTime { value, span } => {
            visitor.on_visit_start_time(value, span);
        }
        Expr::EndTime { value, span } => {
            visitor.on_visit_end_time(value, span);
        }
        Expr::Duration { value, span } => {
            visitor.on_visit_duration(value, span);
        }
        Expr::Code { value, span } => {
            visitor.on_visit_code(value, span);
        }
        Expr::Activity { value, span } => {
            visitor.on_visit_activity(value, span);
        }
        Expr::Entry {
            start,
            end,
            codes,
            duration,
            activity,
            span,
        } => {
            visitor.on_visit_entry(start, end, codes, duration, activity, span);
            walk(start, visitor);
            walk(end, visitor);
            for code in codes {
                walk(code, visitor);
            }
            walk(duration, visitor);
            walk(activity, visitor);
            visitor.on_leave_entry(start, end, codes, duration, activity, span);
        }
        Expr::Journal {
            front_matter,
            lines,
        } => {
            walk(front_matter, visitor);
            for line in lines {
                walk(line, visitor);
            }
        }
        Expr::Error { reason: _, span: _ } => (),
        Expr::NonTargetLine => (),
    }
}
