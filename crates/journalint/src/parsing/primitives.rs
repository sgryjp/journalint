use chrono::NaiveDate;
use chumsky::prelude::*;

use crate::ast;

fn _fixed_length_digits(len: usize) -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(len)
        .at_most(len)
        .collect::<String>()
        .labelled("digits")
}

pub(super) fn date() -> impl Parser<char, ast::LooseDate, Error = Simple<char>> {
    _fixed_length_digits(4)
        .then_ignore(just('-'))
        .then(_fixed_length_digits(2))
        .then_ignore(just('-'))
        .then(_fixed_length_digits(2))
        .try_map(|((y, m), d), span| {
            NaiveDate::from_ymd_opt(
                str::parse::<i32>(&y).unwrap(),
                str::parse::<u32>(&m).unwrap(),
                str::parse::<u32>(&d).unwrap(),
            )
            .map(|d| ast::LooseDate::new(d, span.clone()))
            .ok_or(Simple::custom(
                span,
                format!("invalid date: {:4}-{}-{}", y, m, d),
            ))
        })
}

pub fn time() -> impl Parser<char, ast::LooseTime, Error = Simple<char>> {
    _fixed_length_digits(2)
        .then_ignore(just(':'))
        .then(_fixed_length_digits(2))
        .map_with_span(|(h, s), span| {
            ast::LooseTime::new(
                str::parse::<u32>(&h).unwrap(),
                str::parse::<u32>(&s).unwrap(),
                span,
            )
        })
}

pub fn timerange() -> impl Parser<char, ast::LooseTimeRange, Error = Simple<char>> {
    time()
        .then_ignore(just('-'))
        .then(time())
        .map_with_span(|(s, e), span| ast::LooseTimeRange::new(s, e, span))
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

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;

    #[test]
    fn date() {
        let p = super::date();
        assert_eq!(
            p.parse("2006-01-02").unwrap(),
            ast::LooseDate::new(NaiveDate::from_ymd_opt(2006, 1, 2).unwrap(), 0..10),
        );

        assert!(p.parse("2006-01-00").is_err());
    }

    #[test]
    fn time() {
        let p = super::time();
        assert_eq!(p.parse("01:02").unwrap(), ast::LooseTime::new(1, 2, 0..5));
        assert_eq!(p.parse("24:60").unwrap(), ast::LooseTime::new(24, 60, 0..5));
        assert!(p.parse("24 :60").is_err());
        assert!(p.parse("24: 60").is_err());
    }

    #[test]
    fn timerange() {
        let p = super::timerange();
        assert_eq!(
            p.parse("01:02-03:04"),
            Ok(ast::LooseTimeRange::new(
                ast::LooseTime::new(1, 2, 0..5),
                ast::LooseTime::new(3, 4, 6..11),
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
}
