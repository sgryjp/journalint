use std::ops::Range;
use std::time::Duration;

use chrono::{NaiveDate, NaiveTime};
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

    Time {
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

    fn to_naivetime(&self) -> Result<NaiveTime, JournalintError> {
        NaiveTime::parse_from_str(self.0.as_str(), "%H:%M").map_err(|e| {
            JournalintError::ParseError(format!("unrecognizable time: {e}: {}", self.0))
        })
    }
}

fn front_matter() -> impl Parser<char, Expr, Error = Simple<char>> {
    let delimiter = just('-').repeated().at_least(3);
    let fm_date = just("date").padded().then(just(':').padded()).ignore_then(
        newline()
            .not()
            .repeated()
            .collect::<String>()
            .try_map(|s, span| {
                NaiveDate::parse_from_str(s.as_str(), "%Y-%m-%d")
                    .map_err(|e| Simple::custom(span, format!("unrecognizable date: {e}: {s}")))
            })
            .map_with_span(|value, span| Expr::FrontMatterDate { value, span }),
    );
    let fm_start = just("start").padded().then(just(':').padded()).ignore_then(
        newline()
            .not()
            .repeated()
            .collect::<String>()
            .map_with_span(|value, span| Expr::FrontMatterStartTime {
                value: LooseTime(value),
                span,
            }),
    );
    let fm_end = just("end").padded().then(just(':').padded()).ignore_then(
        newline()
            .not()
            .repeated()
            .collect::<String>()
            .map_with_span(|value, span| Expr::FrontMatterEndTime {
                value: LooseTime(value),
                span,
            }),
    );

    delimiter
        .then(newline())
        .ignore_then(
            fm_date
                .or(fm_start)
                .or(fm_end)
                .then_ignore(newline())
                .repeated(),
        )
        .then_ignore(delimiter)
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
}

fn time() -> impl Parser<char, Expr, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .chain(just(':'))
        .chain::<char, _, _>(filter(|c: &char| c.is_ascii_digit()).repeated().at_least(1))
        .collect::<String>()
        .map_with_span(|string, span| Expr::Time {
            value: LooseTime(string),
            span,
        })
}

fn duration() -> impl Parser<char, Expr, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit() || *c == '.')
        .repeated()
        .collect::<String>()
        .try_map(|s, span: Range<usize>| {
            str::parse::<f64>(&s)
                .map(|n| Expr::Duration {
                    value: Duration::from_secs_f64(n * 3600.0),
                    span: span.clone(),
                })
                .map_err(|e| Simple::custom(span, format!("unrecognizable duration: {e}: {s}")))
        })
}

fn code() -> impl Parser<char, Expr, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|value, span| Expr::Code { value, span })
}

fn activity() -> impl Parser<char, Expr, Error = Simple<char>> {
    newline()
        .not()
        .repeated()
        .collect::<String>()
        .map_with_span(|value, span| Expr::Activity { value, span })
}

fn entry() -> impl Parser<char, Expr, Error = Simple<char>> {
    just('-')
        .ignore_then(time().padded())
        .then_ignore(just('-'))
        .then(time().padded())
        .then(code().padded().repeated().at_most(2))
        .then(duration().padded())
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
    let target_line = entry().then_ignore(newline());
    let non_target_line = newline()
        .not()
        .repeated()
        .then_ignore(newline())
        .to(Expr::NonTargetLine);
    let line = target_line.or(non_target_line);

    front_matter()
        .then(line.repeated())
        .then_ignore(end())
        .map(|(front_matter, lines)| Expr::Journal {
            front_matter: Box::new(front_matter),
            lines,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time() {
        let p = super::time();
        assert_eq!(
            p.parse("01:02").unwrap(),
            Expr::Time {
                value: LooseTime::new("01:02"),
                span: 0..5,
            }
        );
        assert_eq!(
            p.parse("24:60").unwrap(),
            Expr::Time {
                value: LooseTime::new("24:60"),
                span: 0..5,
            }
        );
        assert!(p.parse("24 :60").is_err());
        assert!(p.parse("24: 60").is_err());
    }

    #[test]
    fn duration() {
        let p = super::duration();
        assert_eq!(
            p.parse(".12"),
            Ok(Expr::Duration {
                value: Duration::from_secs(432),
                span: 0..3
            })
        );
        assert_eq!(
            p.parse("12.34"),
            Ok(Expr::Duration {
                value: Duration::from_secs(44424),
                span: 0..5
            })
        );
        assert!(p.parse("1.2.1").is_err());
    }

    #[test]
    fn code() {
        let p = super::code();
        assert_eq!(
            p.parse("X1234567"),
            Ok(Expr::Code {
                value: String::from("X1234567"),
                span: 0..8
            })
        );
        assert_eq!(
            p.parse("014"),
            Ok(Expr::Code {
                value: String::from("014"),
                span: 0..3
            })
        );
    }

    #[test]
    fn activity() {
        let p = super::activity();
        assert_eq!(
            p.parse("foo: bar: baz\n"), // should stop before newline
            Ok(Expr::Activity {
                value: String::from("foo: bar: baz"),
                span: 0..13
            })
        );
    }

    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn entry() {
        let parser = super::entry();
        let (entry, errors) = parser.parse_recovery(EXAMPLE_ENTRY);
        assert_eq!(errors, []);
        assert_eq!(
            entry,
            Some(Expr::Entry {
                start: Box::new(Expr::Time {
                    value: LooseTime::new("09:00"),
                    span: 2..7
                }),
                end: Box::new(Expr::Time {
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
        let p = super::front_matter();
        assert_eq!(
            p.parse(concat!(
                "---\n",
                "date: 2006-01-02\n",
                "start: 15:04\n",
                "end: 24:56\n",
                "---\n"
            ))
            .unwrap(),
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
        assert!(p.parse("date :2006-12-32").is_err());
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
        let (journal, errors) = super::journal().parse_recovery(input);
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
                            start: Box::new(Expr::Time {
                                value: LooseTime::new("09:00"),
                                span: 52..57
                            }),
                            end: Box::new(Expr::Time {
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
