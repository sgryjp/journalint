use std::ops::Range;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chumsky::prelude::*;
use chumsky::text::newline;

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
    fn new<T: Into<String>>(value: T) -> LooseTime {
        LooseTime(value.into())
    }

    pub fn to_datetime(&self, date: &NaiveDate) -> Result<DateTime<Utc>, JournalintError> {
        match NaiveTime::parse_from_str(self.0.as_str(), "%H:%M") {
            Ok(t) => {
                let datetime = NaiveDateTime::new(*date, t);
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
                if h < 24 {
                    return Err(JournalintError::ParseError(format!(
                        "invalid time value: {}: '{}'",
                        e, self.0
                    )));
                }
                if 60 < m {
                    return Err(JournalintError::ParseError(format!(
                        "invalid minute value: {}: '{}'",
                        e, self.0
                    )));
                }
                let time = NaiveTime::from_hms_opt(h - 24, m, 0).unwrap();
                let Some(date) = date.checked_add_days(Days::new(1)) else {
                    return Err(JournalintError::ParseError(format!(
                        "failed to calculate one date ahead of '{}'",
                        date
                    )));
                };
                let datetime = NaiveDateTime::new(date, time);
                Ok(DateTime::from_utc(datetime, Utc))
            }
        }
    }

    pub fn to_naivetime(&self) -> Result<NaiveTime, JournalintError> {
        // TODO: Remove if unused
        NaiveTime::parse_from_str(self.0.as_str(), "%H:%M").map_err(|e| {
            JournalintError::ParseError(format!("unrecognizable time: {e}: {}", self.0))
        })
    }
}

fn front_matter() -> impl Parser<char, Expr, Error = Simple<char>> {
    let delimiter = || just('-').repeated().at_least(3).debug("delimiter");
    let fm_date = || {
        just("date")
            .then_ignore(wsp())
            .then(just(':').then_ignore(wsp()))
            .ignore_then(
                newline()
                    .not()
                    .repeated()
                    .collect::<String>()
                    .try_map(|s, span| {
                        NaiveDate::parse_from_str(s.as_str(), "%Y-%m-%d").map_err(|e| {
                            Simple::custom(span, format!("unrecognizable date: {e}: {s}"))
                        })
                    })
                    .map_with_span(|value, span| Expr::FrontMatterDate { value, span }),
            )
            .debug("fm_date")
    };
    let fm_start = || {
        just("start")
            .then_ignore(wsp())
            .then(just(':').then_ignore(wsp()))
            .ignore_then(
                newline() // TODO: Accept HH:MM formatted string where HH and MM are *digits*
                    .not()
                    .repeated()
                    .collect::<String>()
                    .map_with_span(|value, span| Expr::FrontMatterStartTime {
                        value: LooseTime(value),
                        span,
                    }),
            )
            .debug("fm_start")
    };
    let fm_end = || {
        just("end")
            .then_ignore(wsp())
            .then(just(':').then_ignore(wsp()))
            .ignore_then(
                newline()
                    .not()
                    .repeated()
                    .collect::<String>()
                    .map_with_span(|value, span| Expr::FrontMatterEndTime {
                        value: LooseTime(value),
                        span,
                    }),
            )
            .debug("fm_end")
    };

    delimiter()
        .then(newline())
        .ignore_then(
            fm_date()
                .or(fm_start())
                .or(fm_end())
                .then_ignore(newline())
                .repeated(),
        )
        .then_ignore(delimiter())
        .then_ignore(newline())
        .try_map(|exprs: Vec<Expr>, span| {
            let mut date: Option<Expr> = None;
            let mut start: Option<Expr> = None;
            let mut end: Option<Expr> = None;
            for expr in exprs {
                match &expr {
                    Expr::FrontMatterDate { value: _, span: _ } => {
                        date = Some(expr);
                    }
                    Expr::FrontMatterStartTime { value: _, span: _ } => {
                        start = Some(expr);
                    }
                    Expr::FrontMatterEndTime { value: _, span: _ } => {
                        end = Some(expr);
                    }
                    _ => (),
                }
            }
            let (Some(date), Some(start), Some(end)) = (date, start, end) else {
                return Err(Simple::custom(span, ""));
            };

            Ok(Expr::FrontMatter {
                date: Box::new(date),
                start: Box::new(start),
                end: Box::new(end),
                span,
            })
        })
        .debug("front_matter")
}

fn _time() -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .chain(just(':'))
        .chain::<char, _, _>(filter(|c: &char| c.is_ascii_digit()).repeated().at_least(1))
        .collect::<String>()
}

fn start_time() -> impl Parser<char, Expr, Error = Simple<char>> {
    _time()
        .map_with_span(|string, span| Expr::StartTime {
            value: LooseTime(string),
            span,
        })
        .debug("start_time")
}

fn end_time() -> impl Parser<char, Expr, Error = Simple<char>> {
    _time()
        .map_with_span(|string, span| Expr::EndTime {
            value: LooseTime(string),
            span,
        })
        .debug("end_time")
}

fn duration() -> impl Parser<char, Expr, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit() || *c == '.')
        .repeated()
        .collect::<String>()
        .map_with_span(|s, span| match str::parse::<f64>(&s) {
            Ok(n) => Expr::Duration {
                value: Duration::from_secs_f64(n * 3600.0),
                span,
            },
            Err(e) => Expr::Error {
                reason: format!("unrecognizable duration: {e}: {s}"),
                span,
            },
        })
        .debug("duration")
}

fn code() -> impl Parser<char, Expr, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|value, span| Expr::Code { value, span })
        .debug("code")
}

fn activity() -> impl Parser<char, Expr, Error = Simple<char>> {
    newline()
        .not()
        .repeated()
        .collect::<String>()
        .map_with_span(|value, span| Expr::Activity { value, span })
        .debug("activity")
}

fn entry() -> impl Parser<char, Expr, Error = Simple<char>> {
    just('-')
        .then_ignore(wsp())
        .ignore_then(start_time().then_ignore(just('-')).then(end_time()))
        .then_ignore(wsp())
        .then(code().then_ignore(wsp()).repeated().at_most(2))
        .then(duration().then_ignore(wsp()))
        .then(activity())
        .map_with_span(
            |((((start, end), codes), duration), activity), span| Expr::Entry {
                start: Box::new(start),
                end: Box::new(end),
                codes,
                duration: Box::new(duration),
                activity: Box::new(activity),
                span,
            },
        )
}

fn journal() -> impl Parser<char, Expr, Error = Simple<char>> {
    let target_line = || entry().then_ignore(newline()).debug("target_line");
    let non_target_line = || {
        newline()
            .not()
            .repeated()
            .then_ignore(newline())
            .to(Expr::NonTargetLine)
            .debug("non_target_line")
    };

    front_matter()
        .then(target_line().or(non_target_line()).repeated())
        .then_ignore(end())
        .map(|(front_matter, lines)| Expr::Journal {
            front_matter: Box::new(front_matter),
            lines,
        })
        .debug("journal")
}

// ----------------------------------------------------------------------------
fn wsp() -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_whitespace() && *c != '\r' && *c != '\n')
        .repeated()
        .collect::<String>()
}

// ----------------------------------------------------------------------------
pub fn parse(content: &str) -> (Option<Expr>, Vec<Simple<char>>) {
    journal().parse_recovery(content)
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loose_time_to_datetime() {
        let date1 = NaiveDate::from_ymd_opt(2006, 2, 3).unwrap();
        let date2 = NaiveDate::from_ymd_opt(262143, 12, 31).unwrap();
        // No colon
        assert!(matches!(
            LooseTime::new("2456").to_datetime(&date1),
            Err(JournalintError::ParseError(..))
        ));
        // Too many colons
        assert!(matches!(
            LooseTime::new("2:4:56").to_datetime(&date1),
            Err(JournalintError::ParseError(..))
        ));
        // Non-number hour
        assert!(matches!(
            LooseTime::new("2z:56").to_datetime(&date1),
            Err(JournalintError::ParseError(..))
        ));
        // Non-number minute
        assert!(matches!(
            LooseTime::new("24:5z").to_datetime(&date1),
            Err(JournalintError::ParseError(..))
        ));
        // Not parsable as a time value and its hour is less than 24.
        assert!(matches!(
            LooseTime::new("00:61").to_datetime(&date1),
            Err(JournalintError::ParseError(..))
        ));
        // Loosely valid time value but out of supported range.
        assert!(matches!(
            LooseTime::new("24:56").to_datetime(&date2),
            Err(JournalintError::ParseError(..))
        ));
        // Loosely valid time value which exceeds 23:59.
        assert_eq!(
            LooseTime::new("24:56")
                .to_datetime(&date1)
                .map(|d| d.fixed_offset())
                .ok(),
            DateTime::parse_from_rfc3339("2006-02-04T00:56:00+00:00").ok()
        );
        // Strictly valid time value.
        assert_eq!(
            LooseTime::new("12:34")
                .to_datetime(&date1)
                .map(|d| d.fixed_offset())
                .ok(),
            DateTime::parse_from_rfc3339("2006-02-03T12:34:00+00:00").ok()
        );
    }

    #[test]
    fn _time() {
        let (result, errors) = super::_time().parse_recovery_verbose("01:02");
        assert_eq!(errors, []);
        assert_eq!(result, Some("01:02".to_string()));

        let (result, errors) = super::_time().parse_recovery_verbose("24:60");
        assert_eq!(errors, []);
        assert_eq!(result, Some("24:60".to_string()));

        let (result, errors) = super::_time().parse_recovery_verbose("24 :60");
        assert_eq!(
            errors
                .iter()
                .map(|e| (e.span(), e.to_string()))
                .collect::<Vec<_>>(),
            [(2..3, "found \" \" but expected \":\"".to_string())]
        );
        assert_eq!(result, None);
    }

    #[test]
    fn start_time() {
        let (result, errors) = super::start_time().parse_recovery_verbose("01:02");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::StartTime {
                value: LooseTime("01:02".to_string()),
                span: 0..5
            })
        );
    }

    #[test]
    fn end_time() {
        let (result, errors) = super::end_time().parse_recovery_verbose("01:02");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::EndTime {
                value: LooseTime("01:02".to_string()),
                span: 0..5
            })
        );
    }

    #[test]
    fn duration() {
        let (result, errors) = super::duration().parse_recovery_verbose(".12");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Duration {
                value: Duration::from_secs(432),
                span: 0..3
            })
        );

        let (result, errors) = super::duration().parse_recovery_verbose("12.34");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Duration {
                value: Duration::from_secs(44424),
                span: 0..5
            })
        );

        let input = "1.2.1";
        let (result, errors) = super::duration().parse_recovery_verbose(input);
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Error {
                reason: format!("unrecognizable duration: invalid float literal: {input}"),
                span: 0..5
            })
        );
    }

    #[test]
    fn code() {
        let (result, errors) = super::code().parse_recovery_verbose("X1234567");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Code {
                value: String::from("X1234567"),
                span: 0..8
            })
        );

        let (result, errors) = super::code().parse_recovery_verbose("014");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Code {
                value: String::from("014"),
                span: 0..3
            })
        );
    }

    #[test]
    fn activity() {
        let (result, errors) = super::activity().parse_recovery_verbose("foo: bar: baz\n");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Activity {
                value: String::from("foo: bar: baz"), // should stop before newline
                span: 0..13
            })
        );
    }

    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn entry() {
        let parser = super::entry();
        let (entry, errors) = parser.parse_recovery_verbose(EXAMPLE_ENTRY);
        assert_eq!(errors, []);
        assert_eq!(
            entry,
            Some(Expr::Entry {
                start: Box::new(Expr::StartTime {
                    value: LooseTime::new("09:00"),
                    span: 2..7
                }),
                end: Box::new(Expr::EndTime {
                    value: LooseTime::new("10:15"),
                    span: 8..13
                }),
                codes: vec![
                    Expr::Code {
                        value: "ABCDEFG8".to_string(),
                        span: 14..22
                    },
                    Expr::Code {
                        value: "AB3".to_string(),
                        span: 23..26
                    }
                ],
                duration: Box::new(Expr::Duration {
                    value: Duration::from_secs(3600),
                    span: 27..31
                }),
                activity: Box::new(Expr::Activity {
                    value: "foo: bar: baz".to_string(),
                    span: 32..45
                }),
                span: 0..45
            })
        );
    }

    #[test]
    fn front_matter() {
        let input = concat!(
            "---\n",
            "date: 2006-01-02\n",
            "start: 15:04\n",
            "end: 24:56\n",
            "---\n"
        );

        let (result, errors) = super::front_matter().parse_recovery_verbose(input);
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::FrontMatter {
                date: Box::new(Expr::FrontMatterDate {
                    value: NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(),
                    span: 10..20
                }),
                start: Box::new(Expr::FrontMatterStartTime {
                    value: LooseTime::new("15:04"),
                    span: 28..33
                }),
                end: Box::new(Expr::FrontMatterEndTime {
                    value: LooseTime::new("24:56"),
                    span: 39..44
                }),
                span: 0..49,
            })
        );
    }

    #[test]
    fn journal_basic() {
        let input = format!(
            "---\n\
            date: 2006-01-02\n\
            start: 15:04\n\
            end: 24:56\n\
            ---\n\
            \n\
            {}\n\
            ",
            EXAMPLE_ENTRY
        );
        let (journal, errors) = super::journal().parse_recovery_verbose(input);
        assert_eq!(errors, []);
        assert!(journal.is_some());
        let journal = journal.unwrap();
        match journal {
            Expr::Journal {
                front_matter,
                lines,
            } => {
                assert_eq!(
                    *front_matter,
                    Expr::FrontMatter {
                        date: Box::new(Expr::FrontMatterDate {
                            value: NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(),
                            span: 10..20
                        }),
                        start: Box::new(Expr::FrontMatterStartTime {
                            value: LooseTime::new("15:04"),
                            span: 28..33
                        }),
                        end: Box::new(Expr::FrontMatterEndTime {
                            value: LooseTime::new("24:56"),
                            span: 39..44
                        }),
                        span: 0..49,
                    }
                );

                assert_eq!(
                    lines,
                    vec![
                        Expr::NonTargetLine,
                        Expr::Entry {
                            start: Box::new(Expr::StartTime {
                                value: LooseTime::new("09:00"),
                                span: 52..57
                            }),
                            end: Box::new(Expr::EndTime {
                                value: LooseTime::new("10:15"),
                                span: 58..63
                            }),
                            codes: vec![
                                Expr::Code {
                                    value: "ABCDEFG8".to_string(),
                                    span: 64..72
                                },
                                Expr::Code {
                                    value: "AB3".to_string(),
                                    span: 73..76
                                }
                            ],
                            duration: Box::new(Expr::Duration {
                                value: Duration::from_secs(3600),
                                span: 77..81
                            }),
                            activity: Box::new(Expr::Activity {
                                value: "foo: bar: baz".to_string(),
                                span: 82..95
                            }),
                            span: 50..95
                        }
                    ]
                );
            }
            _ => assert!(false),
        }
    }
}
