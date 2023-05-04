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

fn _entry_line() -> impl Parser<char, Line, Error = Simple<char>> {
    journal_entry().then_ignore(newline()).map(Line::Entry)
}

fn _misc_line() -> impl Parser<char, Line, Error = Simple<char>> {
    newline().not().repeated().map(|_| Line::Misc)
}

pub(crate) fn journal() -> impl Parser<char, Journal, Error = Simple<char>> {
    front_matter()
        .then(_entry_line().or(_misc_line()).repeated())
        .map(|(front_matter, lines)| Journal {
            front_matter,
            entries: lines
                .into_iter()
                .filter_map(|line| match line {
                    Line::Entry(entry) => Some(entry),
                    Line::Misc => None,
                })
                .collect::<Vec<JournalEntry>>(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parsing::primitives::LooseTime;

    #[test]
    fn journal_entry_basic() {
        let input = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";
        let parser = super::journal_entry();
        let entry = parser.parse(input).unwrap();

        assert_eq!(
            entry.time_range,
            LooseTimeRange {
                start: LooseTime {
                    hour: 9,
                    minute: 0,
                    span: 2..7
                },
                end: LooseTime {
                    hour: 10,
                    minute: 15,
                    span: 8..13
                },
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

        assert_eq!(
            entry.duration,
            Duration {
                total_seconds: 3600,
                span: 27..31
            }
        );

        assert_eq!(
            entry.description,
            Description {
                activity: "foo: bar: baz".into(),
                span: 32..45
            }
        );

        assert_eq!(entry.span, 0..45);
    }
}
