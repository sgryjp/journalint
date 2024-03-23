//! Provides parsing logic.
//!
//! See module `ast` for AST related features, and module `lint` for linting logic.
use std::time::Duration;

use chrono::NaiveDate;
use chumsky::{
    error::Simple,
    primitive::{end, filter, just},
    text::newline,
    Parser,
};

use crate::ast::{Expr, LooseTime};

/// Parse a journal file content.
pub fn parse(content: &str) -> (Option<Expr>, Vec<Simple<char>>) {
    journal().parse_recovery(content)
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
                        value: LooseTime::new(value),
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
                        value: LooseTime::new(value),
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
    filter(char::is_ascii_digit)
        .repeated()
        .at_least(1)
        .chain(just(':'))
        .chain::<char, _, _>(filter(char::is_ascii_digit).repeated().at_least(1))
        .collect::<String>()
}

fn start_time() -> impl Parser<char, Expr, Error = Simple<char>> {
    _time()
        .map_with_span(|string, span| Expr::StartTime {
            value: LooseTime::new(string),
            span,
        })
        .debug("start_time")
}

fn end_time() -> impl Parser<char, Expr, Error = Simple<char>> {
    _time()
        .map_with_span(|string, span| Expr::EndTime {
            value: LooseTime::new(string),
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
    filter(|c: &char| !c.is_ascii_whitespace())
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

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::DateTime;
    use rstest::*;

    #[rstest]
    #[case("2456", 2006, 2, 3)] // No colon
    #[case("2:4:56", 2006, 2, 3)] // Too many colons
    #[case("2z:56", 2006, 2, 3)] // Non-number hour
    #[case("24:5z", 2006, 2, 3)] // Non-number minute
    #[case("00:61", 2006, 2, 3)] // Not parsable as a time value and its hour is less than 24.
    #[case("24:56", 999999, 12, 31)] // Loosely valid time value but out of supported range.
    fn loose_time_to_datetime_error(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .or(Some(NaiveDate::MAX))
            .unwrap();

        assert!(matches!(LooseTime::new(input).to_datetime(date), Err(..)));
    }

    #[rstest]
    #[case("24:56", "2006-02-04T00:56:00+00:00")] // Loosely valid time value which exceeds 23:59.
    #[case("50:56", "2006-02-05T02:56:00+00:00")] // Loosely valid time value which exceeds 23:59 (more than two days)
    #[case("12:34", "2006-02-03T12:34:00+00:00")] // Strictly valid time value.
    fn loose_time_to_datetime_normal(#[case] input: &str, #[case] want: &str) {
        let date = NaiveDate::from_ymd_opt(2006, 2, 3).unwrap();

        assert_eq!(
            LooseTime::new(input)
                .to_datetime(date)
                .map(|d| d.fixed_offset())
                .ok(),
            DateTime::parse_from_rfc3339(want).ok()
        );
    }

    #[rstest]
    #[case("01:02", None)]
    #[case("24:60", None)]
    #[case("24 :60", Some([(2..3, "found \" \" but expected \":\"".to_string())]))]
    fn _time(
        #[case] input: &str,
        #[case] expected_errors: Option<[(std::ops::Range<usize>, std::string::String); 1]>,
    ) {
        let (result, errors) = super::_time().parse_recovery_verbose(input);
        if let Some(expected_errors) = expected_errors {
            assert_eq!(
                errors
                    .iter()
                    .map(|e| (e.span(), e.to_string()))
                    .collect::<Vec<_>>(),
                expected_errors
            );
            assert_eq!(result, None);
        } else {
            assert_eq!(errors, []);
            assert_eq!(result, Some(input.to_string()));
        }
    }

    #[test]
    fn start_time() {
        let (result, errors) = super::start_time().parse_recovery_verbose("01:02");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::StartTime {
                value: LooseTime::new("01:02"),
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
                value: LooseTime::new("01:02"),
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

        let (result, errors) = super::code().parse_recovery_verbose("---");
        assert_eq!(errors, []);
        assert_eq!(
            result,
            Some(Expr::Code {
                value: String::from("---"),
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
