use std::ops::Range;

use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;

use super::front_matter::{front_matter, FrontMatter};
use super::primitives::{duration, timerange, Duration, LooseTimeRange};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Journal {
    pub front_matter: FrontMatter,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, PartialEq)]
enum Line {
    Entry(JournalEntry),
    Misc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    pub time_range: LooseTimeRange,
    pub codes: Vec<Code>,
    pub duration: Duration,
    pub description: Description,
    pub span: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    value: String,
    span: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    // TODO: support categories
    activity: String,
    span: Range<usize>,
}

fn _code() -> impl Parser<char, Code, Error = Simple<char>> {
    text::ident().map_with_span(|s, span| Code { value: s, span })
}

fn _description() -> impl Parser<char, Description, Error = Simple<char>> {
    text::newline()
        .not()
        .repeated()
        .collect::<String>()
        .map_with_span(|s, span| Description { activity: s, span })
}

fn journal_entry() -> impl Parser<char, JournalEntry, Error = Simple<char>> {
    just('-')
        .ignore_then(timerange().padded())
        .then(_code().padded().repeated().padded())
        .then(duration().padded())
        .then(_description())
        .map_with_span(
            |(((time_range, codes), duration), description), span| JournalEntry {
                time_range,
                codes,
                duration,
                description,
                span,
            },
        )
}

fn _journal_entry_line() -> impl Parser<char, Line, Error = Simple<char>> {
    journal_entry().then_ignore(newline()).map(Line::Entry)
}

fn _misc_line() -> impl Parser<char, Line, Error = Simple<char>> {
    newline()
        .not()
        .repeated()
        .collect::<String>()
        .then_ignore(newline())
        .map(|_| Line::Misc)
}

fn _line() -> impl Parser<char, Line, Error = Simple<char>> {
    _journal_entry_line().or(_misc_line())
}

pub(crate) fn journal() -> impl Parser<char, Journal, Error = Simple<char>> {
    front_matter()
        .then(_line().repeated().then_ignore(end()))
        .map(|(front_matter, lines)| Journal {
            front_matter,
            entries: lines
                .iter()
                .filter_map(|line| match line {
                    Line::Entry(entry) => Some(entry.clone()), // TODO: Inefficient...
                    Line::Misc => None,
                })
                .collect::<Vec<JournalEntry>>(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parsing::primitives::{LooseDate, LooseTime};

    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn journal_entry() {
        let parser = super::journal_entry();
        let entry = parser.parse(EXAMPLE_ENTRY).unwrap();

        assert_eq!(
            entry.time_range,
            LooseTimeRange {
                start: LooseTime::new_hm(9, 0, 2..7),
                end: LooseTime::new_hm(10, 15, 8..13),
                span: 2..13
            }
        );

        assert_eq!(
            entry.codes,
            vec![
                Code {
                    value: "ABCDEFG8".into(),
                    span: 14..22
                },
                Code {
                    value: "AB3".into(),
                    span: 23..26
                }
            ],
        );

        assert_eq!(entry.duration, Duration::new(3600, 27..31));

        assert_eq!(
            entry.description,
            Description {
                activity: "foo: bar: baz".into(),
                span: 32..45
            }
        );

        assert_eq!(entry.span, 0..45);
    }

    #[test]
    fn _misc_line() {
        let (result, errors) = super::_misc_line().parse_recovery_verbose("\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(Line::Misc));

        let (result, errors) = super::_misc_line().parse_recovery_verbose("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(Line::Misc));
    }

    #[test]
    fn _line() {
        let (result, errors) = super::_line().parse_recovery_verbose("");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, None);

        let (result, errors) = super::_line().parse_recovery_verbose("\n");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, Some(Line::Misc));

        let (result, errors) = super::_line().parse_recovery_verbose("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(Line::Misc));

        let (result, errors) =
            super::_line().parse_recovery_verbose(format!("{}\n", EXAMPLE_ENTRY));
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(matches!(result, Some(Line::Entry(_))));
    }

    #[test]
    fn journal() {
        let input = format!(
            "---
date: 2006-01-02
start: 15:04
---

{}
",
            EXAMPLE_ENTRY
        );
        let (journal, errors) = super::journal().parse_recovery_verbose(input);
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(matches!(journal, Some(_)));
        assert_eq!(
            journal.map(|j| j.front_matter),
            Some(FrontMatter {
                date: LooseDate::new_ymd(2006, 1, 2, 10..20),
                start_time: Some(LooseTime::new_hm(15, 4, 28..33)),
                end_time: None,
            })
        );
    }
}
