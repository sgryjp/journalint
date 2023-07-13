use chumsky::prelude::*;
use chumsky::primitive::just;
use chumsky::text::newline;
use chumsky::Parser;

use crate::ast;

fn _front_matter_date() -> impl Parser<char, ast::FrontMatterItem, Error = Simple<char>> {
    just("date")
        .then(just(':').padded())
        .ignore_then(date())
        .map(ast::FrontMatterItem::Date)
}

fn _front_matter_start_time() -> impl Parser<char, ast::FrontMatterItem, Error = Simple<char>> {
    just("start")
        .then(just(':').padded())
        .ignore_then(time())
        .map(ast::FrontMatterItem::StartTime)
}

fn _front_matter_end_time() -> impl Parser<char, ast::FrontMatterItem, Error = Simple<char>> {
    just("end")
        .then(just(':').padded())
        .ignore_then(time())
        .map(ast::FrontMatterItem::EndTime)
}

fn _front_matter_item() -> impl Parser<char, ast::FrontMatterItem, Error = Simple<char>> {
    _front_matter_date()
        .or(_front_matter_start_time())
        .or(_front_matter_end_time())
}

pub fn front_matter() -> impl Parser<char, ast::FrontMatter, Error = Simple<char>> {
    let delimiter = just('-').repeated().at_least(3);
    delimiter
        .then(text::newline().repeated())
        .ignore_then(
            _front_matter_item()
                .then_ignore(text::newline().repeated())
                .repeated(),
        )
        .try_map(|items, span| {
            let mut date: Option<ast::Date> = None;
            let mut start: Option<ast::Time> = None;
            let mut end: Option<ast::Time> = None;
            for item in items {
                match item {
                    ast::FrontMatterItem::Date(d) => date = Some(d),
                    ast::FrontMatterItem::StartTime(t) => start = Some(t),
                    ast::FrontMatterItem::EndTime(t) => end = Some(t),
                }
            }
            let Some(date) = date else {
                return Err(Simple::custom(
                    span,
                    "date not found in the front matter".to_string(),
                ));
            };
            Ok(ast::FrontMatter::new(date, start, end))
        })
}

fn _fixed_length_digits(len: usize) -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(len)
        .at_most(len)
        .collect::<String>()
        .labelled("digits")
}

pub fn date() -> impl Parser<char, ast::Date, Error = Simple<char>> {
    _fixed_length_digits(4)
        .chain::<char, _, _>(just('-'))
        .chain::<char, _, _>(_fixed_length_digits(2))
        .chain::<char, _, _>(just('-'))
        .chain::<char, _, _>(_fixed_length_digits(2))
        .collect::<String>()
        .map_with_span(ast::Date::new)
}

pub fn time() -> impl Parser<char, ast::Time, Error = Simple<char>> {
    _fixed_length_digits(2)
        .then_ignore(just(':'))
        .then(_fixed_length_digits(2))
        .map_with_span(|(h, s), span| {
            ast::Time::new(
                str::parse::<u32>(&h).unwrap(),
                str::parse::<u32>(&s).unwrap(),
                span,
            )
        })
}

pub fn timerange() -> impl Parser<char, ast::TimeRange, Error = Simple<char>> {
    time()
        .then_ignore(just('-'))
        .then(time())
        .map_with_span(|(s, e), span| ast::TimeRange::new(s, e, span))
}

pub fn duration() -> impl Parser<char, ast::Duration, Error = Simple<char>> {
    _fixed_length_digits(1)
        .then_ignore(just('.'))
        .then(_fixed_length_digits(2))
        .map(|(a, b)| format!("{}.{}", a, b))
        .try_map(|s, span| {
            str::parse::<f64>(&s).map_err(|e| Simple::custom(span, format!("{}", e)))
        })
        .map_with_span(|n, span| ast::Duration::from_secs((n * 3600.0) as u64, span))
}

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
    use std::ops::Range;

    use super::*;

    //                           0---------1---------2---------3---------4---45
    const EXAMPLE_ENTRY: &str = "- 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz";

    #[test]
    fn front_matter_date() {
        let p = super::_front_matter_date();
        assert_eq!(
            p.parse("date: 2006-01-02").unwrap(),
            ast::FrontMatterItem::Date(ast::Date::new("2006-01-02", 6..16)),
        );
        assert!(p.parse("date :2006-012-02").is_err());
    }

    #[test]
    fn front_matter_start_time() {
        let p = super::_front_matter_start_time();
        assert_eq!(
            p.parse("start: 24:56").unwrap(),
            ast::FrontMatterItem::StartTime(ast::Time::new(24, 56, 7..12))
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }

    #[test]
    fn front_matter_end_time() {
        let p = super::_front_matter_end_time();
        assert_eq!(
            p.parse("end: 24:56").unwrap(),
            ast::FrontMatterItem::EndTime(ast::Time::new(24, 56, 5..10))
        );
        assert!(p.parse("date :2006-12-32").is_err());
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
            ast::FrontMatter::new(
                ast::Date::new("2006-01-02", 10..20),
                Some(ast::Time::new(15, 4, 28..33)),
                Some(ast::Time::new(24, 56, 39..44))
            )
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }

    #[test]
    fn date() {
        let p = super::date();
        assert_eq!(
            p.parse("2006-01-02").unwrap(),
            ast::Date::new("2006-01-02", 0..10)
        );
    }

    #[test]
    fn time() {
        let p = super::time();
        assert_eq!(p.parse("01:02").unwrap(), ast::Time::new(1, 2, 0..5));
        assert_eq!(p.parse("24:60").unwrap(), ast::Time::new(24, 60, 0..5));
        assert!(p.parse("24 :60").is_err());
        assert!(p.parse("24: 60").is_err());
    }

    #[test]
    fn timerange() {
        let p = super::timerange();
        assert_eq!(
            p.parse("01:02-03:04"),
            Ok(ast::TimeRange::new(
                ast::Time::new(1, 2, 0..5),
                ast::Time::new(3, 4, 6..11),
                Range { start: 0, end: 11 },
            ))
        );
        assert!(p.parse("01 :02-03:04").is_err());
        assert!(p.parse("01: 02-03:04").is_err());
        assert!(p.parse("01:02 -03:04").is_err());
        assert!(p.parse("01:02- 03:04").is_err());
        assert!(p.parse("01:02-03 :04").is_err());
        assert!(p.parse("01:02-03: 04").is_err());
    }

    #[test]
    fn duration() {
        let p = super::duration();
        assert!(p.parse(".12").is_err());
        assert_eq!(
            p.parse("1.23").unwrap(),
            ast::Duration::from_secs(4428, 0..4)
        );
        assert!(p.parse("12.34").is_err());
        assert!(p.parse("1.2").is_err());
        assert_eq!(
            p.parse("1.23").unwrap(),
            ast::Duration::from_secs(4428, 0..4)
        );
        assert_eq!(
            p.parse("1.234").unwrap(),
            ast::Duration::from_secs(4428, 0..4)
        );

        assert!(p.parse("1.2").is_err());
        assert!(p.parse(".123").is_err());
    }

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
        let (entry, errors) = parser.parse_recovery(EXAMPLE_ENTRY);
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert!(entry.is_some());
        let entry = entry.unwrap();

        assert_eq!(
            *entry.time_range(),
            ast::TimeRange::new(
                ast::Time::new(9, 0, 2..7),
                ast::Time::new(10, 15, 8..13),
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
        let (result, errors) = super::_misc_line().parse_recovery("\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) = super::_misc_line().parse_recovery("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));
    }

    #[test]
    fn _line() {
        let (result, errors) = super::_line().parse_recovery("");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, None);

        let (result, errors) = super::_line().parse_recovery("\n");
        errors.iter().for_each(|e| println!("{:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) = super::_line().parse_recovery("- 09:00-10:15\n");
        errors.iter().for_each(|e| println!("!! {:?}", e));
        assert_eq!(result, Some(ast::Line::Misc));

        let (result, errors) = super::_line().parse_recovery(format!("{}\n", EXAMPLE_ENTRY));
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
        let (journal, errors) = super::journal().parse_recovery(input);
        assert_eq!(errors, []);
        assert!(journal.is_some());
        let journal = journal.unwrap();
        assert_eq!(
            *journal.front_matter(),
            ast::FrontMatter::new(
                ast::Date::new("2006-01-02", 10..20),
                Some(ast::Time::new(15, 4, 28..33)),
                None,
            )
        );
    }
}
