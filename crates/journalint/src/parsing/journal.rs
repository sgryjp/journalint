use chumsky::prelude::*;
use chumsky::text::newline;
use chumsky::Parser;

use super::front_matter::front_matter;
use super::primitives::{duration, timerange};
use crate::ast;

fn _code() -> impl Parser<char, ast::Code, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(ast::Code::new)
}

fn _description() -> impl Parser<char, ast::Description, Error = Simple<char>> {
    text::newline()
        .not()
        .repeated()
        .collect::<String>()
        .map_with_span(ast::Description::new)
}

fn journal_entry() -> impl Parser<char, ast::JournalEntry, Error = Simple<char>> {
    just('-')
        .ignore_then(timerange().padded())
        .then(_code().padded().repeated().at_least(2).at_most(2))
        .then(duration().padded())
        .then(_description())
        .map_with_span(|(((time_range, codes), duration), description), span| {
            ast::JournalEntry::new(time_range, codes, duration, description, span)
        })
}

fn _journal_entry_line() -> impl Parser<char, ast::Line, Error = Simple<char>> {
    journal_entry().then_ignore(newline()).map(ast::Line::Entry)
}

fn _misc_line() -> impl Parser<char, ast::Line, Error = Simple<char>> {
    newline()
        .not()
        .repeated()
        .collect::<String>()
        .then_ignore(newline())
        .map(|_| ast::Line::Misc)
}

fn _line() -> impl Parser<char, ast::Line, Error = Simple<char>> {
    _journal_entry_line().or(_misc_line())
}

pub(crate) fn journal() -> impl Parser<char, ast::Journal, Error = Simple<char>> {
    front_matter()
        .then(_line().repeated().then_ignore(end()))
        .map(|(front_matter, lines)| {
            ast::Journal::new(
                front_matter,
                lines
                    .iter()
                    .filter_map(|line| match line {
                        ast::Line::Entry(entry) => Some(entry.clone()), // TODO: Inefficient...
                        ast::Line::Misc => None,
                    })
                    .collect::<Vec<ast::JournalEntry>>(),
            )
        })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    //                           0---------1---------2---------3---------4---45
    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn code() {
        let parser = super::_code();

        let result = parser.parse("X1234567");
        assert_eq!(result, Ok(ast::Code::new(String::from("X1234567"), 0..8,)));

        let result = parser.parse("014");
        assert_eq!(result, Ok(ast::Code::new(String::from("014"), 0..3,)));
    }
    #[test]
    fn journal_entry() {
        let parser = super::journal_entry();
        let (entry, errors) = parser.parse_recovery_verbose(EXAMPLE_ENTRY);
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(entry.is_some());
        let entry = entry.unwrap();

        assert_eq!(
            *entry.time_range(),
            ast::LooseTimeRange::new(
                ast::LooseTime::new(9, 0, 2..7),
                ast::LooseTime::new(10, 15, 8..13),
                2..13
            )
        );

        assert_eq!(
            entry.codes(),
            vec![
                ast::Code::new("ABCDEFG8".into(), 14..22),
                ast::Code::new("AB3".into(), 23..26)
            ],
        );

        assert_eq!(*entry.duration(), ast::Duration::from_secs(3600, 27..31));

        assert_eq!(
            *entry.description(),
            ast::Description::new("foo: bar: baz".into(), 32..45)
        );

        assert_eq!(*entry.span(), 0..45);
    }

    #[test]
    fn _misc_line() {
        let (result, errors) = super::_misc_line().parse_recovery_verbose("\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) = super::_misc_line().parse_recovery_verbose("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));
    }

    #[test]
    fn _line() {
        let (result, errors) = super::_line().parse_recovery_verbose("");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, None);

        let (result, errors) = super::_line().parse_recovery_verbose("\n");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) = super::_line().parse_recovery_verbose("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) =
            super::_line().parse_recovery_verbose(format!("{}\n", EXAMPLE_ENTRY));
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(matches!(result, Some(ast::Line::Entry(_))));
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
        assert_eq!(errors, []);
        assert!(journal.is_some());
        let journal = journal.unwrap();
        assert_eq!(
            *journal.front_matter(),
            ast::FrontMatter::new(
                ast::LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 10..20),
                Some(ast::LooseTime::new(15, 4, 28..33)),
                None,
            )
        );
    }
}
