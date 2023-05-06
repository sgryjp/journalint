use chumsky::prelude::*;
use chumsky::primitive::just;
use chumsky::Parser;

use super::primitives::date;
use super::primitives::time;
use super::primitives::LooseDate;
use super::primitives::LooseTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontMatter {
    date: LooseDate,
    start: Option<LooseTime>,
    end: Option<LooseTime>,
}

impl FrontMatter {
    pub fn new(
        date: LooseDate,
        start_time: Option<LooseTime>,
        end_time: Option<LooseTime>,
    ) -> Self {
        Self {
            date,
            start: start_time,
            end: end_time,
        }
    }

    pub fn date(&self) -> &LooseDate {
        &self.date
    }

    pub fn start(&self) -> Option<&LooseTime> {
        self.start.as_ref()
    }

    pub fn end(&self) -> Option<&LooseTime> {
        self.end.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FrontMatterItem {
    Date(LooseDate),
    StartTime(LooseTime),
    EndTime(LooseTime),
}

fn _front_matter_date() -> impl Parser<char, FrontMatterItem, Error = Simple<char>> {
    just("date")
        .then(just(':').padded())
        .ignore_then(date())
        .map(FrontMatterItem::Date)
}

fn _front_matter_start_time() -> impl Parser<char, FrontMatterItem, Error = Simple<char>> {
    just("start")
        .then(just(':').padded())
        .ignore_then(time())
        .map(FrontMatterItem::StartTime)
}

fn _front_matter_end_time() -> impl Parser<char, FrontMatterItem, Error = Simple<char>> {
    just("end")
        .then(just(':').padded())
        .ignore_then(time())
        .map(FrontMatterItem::EndTime)
}

fn _front_matter_item() -> impl Parser<char, FrontMatterItem, Error = Simple<char>> {
    _front_matter_date()
        .or(_front_matter_start_time())
        .or(_front_matter_end_time())
}

pub(super) fn front_matter() -> impl Parser<char, FrontMatter, Error = Simple<char>> {
    let delimiter = just('-').repeated().at_least(3);
    delimiter
        .then(text::newline().repeated())
        .ignore_then(
            _front_matter_item()
                .then_ignore(text::newline().repeated())
                .repeated(),
        )
        .try_map(|items, span| {
            let mut date: Option<LooseDate> = None;
            let mut start_time: Option<LooseTime> = None;
            let mut end_time: Option<LooseTime> = None;
            for item in items {
                match item {
                    FrontMatterItem::Date(d) => date = Some(d),
                    FrontMatterItem::StartTime(t) => start_time = Some(t),
                    FrontMatterItem::EndTime(t) => end_time = Some(t),
                }
            }
            let Some(date) = date else {
                return Err(Simple::custom(span, "date not found in the front matter".to_string()))
            };
            Ok(FrontMatter {
                date,
                start: start_time,
                end: end_time,
            })
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
            FrontMatterItem::Date(LooseDate::new(
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
            FrontMatterItem::StartTime(LooseTime::new(24, 56, 7..12))
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }

    #[test]
    fn front_matter_end_time() {
        let p = super::_front_matter_end_time();
        assert_eq!(
            p.parse("end: 24:56").unwrap(),
            FrontMatterItem::EndTime(LooseTime::new(24, 56, 5..10))
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
            FrontMatter {
                date: LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 10..20),
                start: Some(LooseTime::new(15, 4, 28..33)),
                end: Some(LooseTime::new(24, 56, 39..44))
            }
        );
        assert!(p.parse("date :2006-12-32").is_err());
    }
}
