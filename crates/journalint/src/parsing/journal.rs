use std::ops::Range;

use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;

use super::front_matter::{front_matter, FrontMatter};
use super::primitives::{duration, timerange, Duration, LooseTimeRange};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Journal {
    front_matter: FrontMatter,
    entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn new(front_matter: FrontMatter, entries: Vec<JournalEntry>) -> Self {
        Self {
            front_matter,
            entries,
        }
    }

    pub fn front_matter(&self) -> &FrontMatter {
        &self.front_matter
    }

    pub fn entries(&self) -> &[JournalEntry] {
        self.entries.as_ref()
    }
}

#[derive(Debug, PartialEq)]
enum Line {
    Entry(JournalEntry),
    Misc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    time_range: LooseTimeRange,
    codes: Vec<Code>,
    duration: Duration,
    description: Description,
    span: Range<usize>,
}

impl JournalEntry {
    pub fn new(
        time_range: LooseTimeRange,
        codes: Vec<Code>,
        duration: Duration,
        description: Description,
        span: Range<usize>,
    ) -> Self {
        Self {
            time_range,
            codes,
            duration,
            description,
            span,
        }
    }

    pub fn time_range(&self) -> &LooseTimeRange {
        &self.time_range
    }

    pub fn codes(&self) -> &[Code] {
        self.codes.as_ref()
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn description(&self) -> &Description {
        &self.description
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
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
    filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|s, span| Code { value: s, span })
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
        .then(_code().padded().repeated().at_least(2).at_most(2))
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
    use chrono::NaiveDate;

    use super::super::primitives::{LooseDate, LooseTime};
    use super::*;

    //                           0---------1---------2---------3---------4---45
    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn code() {
        let parser = super::_code();

        let result = parser.parse("X1234567");
        assert_eq!(
            result,
            Ok(Code {
                value: String::from("X1234567"),
                span: 0..8,
            })
        );

        let result = parser.parse("014");
        assert_eq!(
            result,
            Ok(Code {
                value: String::from("014"),
                span: 0..3,
            })
        );
    }
    #[test]
    fn journal_entry() {
        let parser = super::journal_entry();
        let (entry, errors) = parser.parse_recovery_verbose(EXAMPLE_ENTRY);
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(entry.is_some());
        let entry = entry.unwrap();

        assert_eq!(
            entry.time_range,
            LooseTimeRange::new(
                LooseTime::new(9, 0, 2..7),
                LooseTime::new(10, 15, 8..13),
                2..13
            )
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

        assert_eq!(entry.duration, Duration::from_secs(3600, 27..31));

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
            "---\n\
            date: 2006-01-02\n\
            start: 15:04\n\
            ---\n\
            \n\
            {}\n\
            ",
            EXAMPLE_ENTRY
        );
        let (journal, errors) = super::journal().parse_recovery_verbose(input);
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(matches!(journal, Some(_)));
        assert_eq!(
            journal.map(|j| j.front_matter),
            Some(FrontMatter::new(
                LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 10..20),
                Some(LooseTime::new(15, 4, 28..33)),
                None,
            ))
        );
    }
}
