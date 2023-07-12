use chumsky::prelude::*;
use chumsky::primitive::just;
use chumsky::Parser;

use super::primitives::date;
use super::primitives::time;
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

pub(super) fn front_matter() -> impl Parser<char, ast::FrontMatter, Error = Simple<char>> {
    let delimiter = just('-').repeated().at_least(3);
    delimiter
        .then(text::newline().repeated())
        .ignore_then(
            _front_matter_item()
                .then_ignore(text::newline().repeated())
                .repeated(),
        )
        .try_map(|items, span| {
            let mut date: Option<ast::LooseDate> = None;
            let mut start: Option<ast::LooseTime> = None;
            let mut end: Option<ast::LooseTime> = None;
            for item in items {
                match item {
                    ast::FrontMatterItem::Date(d) => date = Some(d),
                    ast::FrontMatterItem::StartTime(t) => start = Some(t),
                    ast::FrontMatterItem::EndTime(t) => end = Some(t),
                }
            }
            let Some(date) = date else {
                return Err(Simple::custom(span, "date not found in the front matter".to_string()))
            };
            Ok(ast::FrontMatter::new(date, start, end))
        })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn front_matter_date() {
        let p = super::_front_matter_date();
        assert_eq!(
            p.parse("date: 2006-01-02").unwrap(),
            ast::FrontMatterItem::Date(ast::LooseDate::new(
                NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(),
                6..16
            )),
        );
        assert!(p.parse("date :2006-012-02").is_err());
    }

    #[test]
    fn front_matter_start_time() {
        let p = super::_front_matter_start_time();
        assert_eq!(
            p.parse("start: 24:56").unwrap(),
            ast::FrontMatterItem::StartTime(ast::LooseTime::new(24, 56, 7..12))
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }

    #[test]
    fn front_matter_end_time() {
        let p = super::_front_matter_end_time();
        assert_eq!(
            p.parse("end: 24:56").unwrap(),
            ast::FrontMatterItem::EndTime(ast::LooseTime::new(24, 56, 5..10))
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
                ast::LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 10..20),
                Some(ast::LooseTime::new(15, 4, 28..33)),
                Some(ast::LooseTime::new(24, 56, 39..44))
            )
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }
}
