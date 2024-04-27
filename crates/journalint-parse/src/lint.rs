//! Provides lint logic.
//!
//! See module `ast` for AST related features, and module `parse` for parsing logic.
use std::ops::Range;
use std::option::Option;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use url::Url;

use crate::ast::{walk, Expr, LooseTime, Visitor};
use crate::diagnostic::{Diagnostic, DiagnosticRelatedInformation};
use crate::violation::Violation;

pub struct Linter<'a> {
    source: &'a Url,
    diagnostics: Vec<Diagnostic>,

    fm_date: Option<(NaiveDate, Range<usize>)>,
    fm_start: Option<(LooseTime, Range<usize>)>,
    fm_start_value: Option<DateTime<Utc>>,
    fm_end: Option<(LooseTime, Range<usize>)>,
    fm_end_value: Option<DateTime<Utc>>,
    seen_first_entry_start: bool,

    entry_start: Option<(DateTime<Utc>, Range<usize>)>,
    entry_end: Option<(DateTime<Utc>, Range<usize>)>,
    prev_entry_end: Option<(DateTime<Utc>, Range<usize>)>,
}

impl<'a> Linter<'a> {
    pub fn new(source: &Url) -> Linter {
        Linter {
            source,
            diagnostics: vec![],

            fm_date: None,
            fm_start: None,
            fm_start_value: None,
            fm_end: None,
            fm_end_value: None,
            seen_first_entry_start: false,

            entry_start: None,
            entry_end: None,
            prev_entry_end: None,
        }
    }

    /// Check if date value in the front matter does not match the one in the filename
    fn check_fm_date_matches_filename(&mut self, value: &NaiveDate, span: &Range<usize>) {
        if let Some(stem) = PathBuf::from(self.source.path())
            .file_stem()
            .and_then(|s| s.to_str())
        {
            if let Ok(date_in_filename) = NaiveDate::parse_from_str(stem, "%Y-%m-%d") {
                if date_in_filename != *value {
                    let expectation = date_in_filename.format("%Y-%m-%d").to_string();
                    self.diagnostics.push(Diagnostic::new_warning(
                        span.clone(),
                        Violation::MismatchedDates,
                        format!(
                            "Date is different from the one in the filename: expected to be {}",
                            expectation.as_str()
                        ),
                        None,
                    ));
                }
            }
        }
    }

    /// Check if start time in the front matter does not match the first entry's start time.
    fn check_fm_start_matches_first_entry(&mut self, value: &LooseTime, _span: &Range<usize>) {
        debug_assert!(self.seen_first_entry_start);

        let Some((fm_start_time, fm_start_span)) = self.fm_start.as_ref() else {
            return;
        };

        if fm_start_time != value {
            self.diagnostics.push(Diagnostic::new_warning(
                fm_start_span.clone(),
                Violation::MismatchedStartTime,
                format!(
                    "Start time is different from the one of the first entry: expected to be {}.",
                    value.as_str()
                ),
                None,
            ));
        }
    }

    fn check_fm_date_exists(&mut self, span: &Range<usize>) {
        if self.fm_date.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Violation::MissingDate,
                "Field 'date' is missing".to_string(),
                None,
            ));
        }
    }

    fn check_fm_start_exists(&mut self, span: &Range<usize>) {
        if self.fm_start.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Violation::MissingStartTime,
                "Field 'start' is missing".to_string(),
                None,
            ));
        }
    }

    fn check_fm_start_is_valid(&mut self) -> Option<DateTime<Utc>> {
        let (date, _) = self.fm_date.as_ref()?;
        let (start, start_span) = self.fm_start.as_ref()?;

        match start.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    start_span.clone(),
                    Violation::InvalidStartTime,
                    format!("Invalid start time: {e}"),
                    None,
                ));
                None
            }
        }
    }

    fn check_fm_end_exists(&mut self, span: &Range<usize>) {
        if self.fm_end.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Violation::MissingEndTime,
                "Field 'end' is missing".to_string(),
                None,
            ));
        }
    }

    fn check_fm_end_is_valid(&mut self) -> Option<DateTime<Utc>> {
        let (date, _) = self.fm_date.as_ref()?;
        let (end, end_span) = self.fm_end.as_ref()?;

        match end.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    end_span.clone(),
                    Violation::InvalidEndTime,
                    format!("Invalid end time: {e}"),
                    None,
                ));
                None
            }
        }
    }

    // Check if start time matches the end of the previous entry
    fn check_prev_end_equals_next_start(&mut self, start_dt: DateTime<Utc>, span: &Range<usize>) {
        if let Some((prev_end_dt, prev_end_range)) = self.prev_entry_end.as_ref() {
            if start_dt != *prev_end_dt {
                let expectation = prev_end_dt.format("%H:%M").to_string();
                self.diagnostics.push(Diagnostic::new_warning(
                    span.clone(),
                    Violation::TimeJumped,
                    format!("The start time does not match the previous entry's end time, which is {expectation}"),
                    Some(vec![DiagnosticRelatedInformation::new(
                        self.source.clone(),
                        prev_end_range.clone(),
                        format!(
                            "Previous entry's end time is {:02}:{:02}",
                            prev_end_dt.hour(),
                            prev_end_dt.minute()
                        ),
                    )]),
                ));
            }
        }
    }

    fn check_start_time(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Option<DateTime<Utc>> {
        let (date, _) = self.fm_date.as_ref()?;

        match value.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                // Start time is not a valid value
                self.diagnostics.push(Diagnostic::new_warning(
                    span.clone(),
                    Violation::InvalidStartTime,
                    format!("Invalid start time: {e}"),
                    None,
                ));
                None
            }
        }
    }

    fn check_end_time(&mut self, value: &LooseTime, span: &Range<usize>) -> Option<DateTime<Utc>> {
        let (date, _) = self.fm_date.as_ref()?;

        match value.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    span.clone(),
                    Violation::InvalidEndTime,
                    format!("Invalid end time: {e}"),
                    None,
                ));
                None
            }
        }
    }

    fn check_end_time_exceeds_start_time(&mut self) {
        let Some((start, _)) = self.entry_start.as_ref() else {
            return;
        };
        let Some((end, end_span)) = self.entry_end.as_ref() else {
            return;
        };

        let Ok(_) = (*end - *start).to_std() else {
            self.diagnostics.push(Diagnostic::new_warning(
                end_span.clone(),
                Violation::NegativeTimeRange,
                format!(
                    "End time is not ahead of start time ({})",
                    start.format("%H:%M"),
                ),
                None,
            ));
            return;
        };
    }

    fn check_duration_matches_end_minus_start(&mut self, value: &Duration, span: &Range<usize>) {
        let Some((start, _)) = self.entry_start.as_ref() else {
            return;
        };
        let Some((end, _)) = self.entry_end.as_ref() else {
            return;
        };
        let Ok(calculated) = (*end - *start).to_std() else {
            return;
        };
        let written = value;
        if calculated != *written {
            let expectation = calculated.as_secs_f64() / 3600.0;
            let expectation = format!("{expectation:1.2}");
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Violation::IncorrectDuration,
                format!("Incorrect duration: expected {expectation}"),
                None,
            ));
        }
    }
}

impl Visitor<()> for Linter<'_> {
    fn on_visit_fm_date(&mut self, value: &NaiveDate, span: &Range<usize>) -> Result<(), ()> {
        self.fm_date = Some((*value, span.clone()));
        self.check_fm_date_matches_filename(value, span);
        Ok(())
    }

    fn on_visit_fm_start(&mut self, value: &LooseTime, span: &Range<usize>) -> Result<(), ()> {
        self.fm_start = Some((value.clone(), span.clone()));
        Ok(())
    }

    fn on_visit_fm_end(&mut self, value: &LooseTime, span: &Range<usize>) -> Result<(), ()> {
        self.fm_end = Some((value.clone(), span.clone()));
        Ok(())
    }

    fn on_leave_fm(&mut self, span: &Range<usize>) -> Result<(), ()> {
        // Calculate exact time of start and end
        self.fm_start_value = self.check_fm_start_is_valid();
        self.fm_end_value = self.check_fm_end_is_valid();

        // Warn if one of date, start and end is missing
        self.check_fm_date_exists(span);
        self.check_fm_start_exists(span);
        self.check_fm_end_exists(span);

        Ok(())
    }

    fn on_visit_start_time(&mut self, value: &LooseTime, span: &Range<usize>) -> Result<(), ()> {
        if let Some(start_dt) = self.check_start_time(value, span) {
            self.entry_start = Some((start_dt, span.clone()));
            self.check_prev_end_equals_next_start(start_dt, span);
        }

        if !self.seen_first_entry_start {
            self.seen_first_entry_start = true;
            self.check_fm_start_matches_first_entry(value, span);
        }

        Ok(())
    }

    fn on_visit_end_time(&mut self, value: &LooseTime, span: &Range<usize>) -> Result<(), ()> {
        if let Some(dt) = self.check_end_time(value, span) {
            self.entry_end = Some((dt, span.clone()));
        }
        Ok(())
    }

    fn on_visit_duration(&mut self, value: &Duration, span: &Range<usize>) -> Result<(), ()> {
        self.check_end_time_exceeds_start_time();
        self.check_duration_matches_end_minus_start(value, span);
        Ok(())
    }

    fn on_leave_entry(&mut self, _span: &Range<usize>) -> Result<(), ()> {
        self.entry_start = None;
        self.prev_entry_end = self.entry_end.take();
        Ok(())
    }

    fn on_leave_journal(&mut self) -> Result<(), ()> {
        // Compare end-time of front-matter and one of the last entry.
        if let (
            Some((_, fm_end_span)),
            Some(fm_end_value),
            Some((last_entry_end_value, last_entry_end_span)),
        ) = (&self.fm_end, &self.fm_end_value, &self.prev_entry_end)
        {
            if fm_end_value != last_entry_end_value {
                // Emit violation to front-matter
                self.diagnostics.push(Diagnostic::new_warning(
                    fm_end_span.clone(),
                    Violation::MismatchedEndTime,
                    format!(
                        "End time in the front-matter is different from the one of the last \
                         entry: expected to be {}.",
                        last_entry_end_value.format("%H:%M")
                    ),
                    Some(vec![DiagnosticRelatedInformation::new(
                        self.source.clone(),
                        last_entry_end_span.clone(),
                        format!(
                            "The last entry ends with {}.",
                            last_entry_end_value.format("%H:%M")
                        ),
                    )]),
                ));
            }
        }
        Ok(())
    }
}

pub fn lint(journal: &Expr, url: &Url) -> Vec<Diagnostic> {
    let mut visitor = Linter::new(url);
    walk(journal, &mut visitor).expect("walk for linting expected to succeed always.");
    visitor.diagnostics
}

pub fn parse_and_lint(url: &Url, content: &str) -> (Option<Expr>, Vec<Diagnostic>) {
    let (journal, parse_errors) = crate::parse::parse(content);
    let mut diagnostics: Vec<Diagnostic> = parse_errors.iter().map(Diagnostic::from).collect();
    if let Some(journal) = &journal {
        let mut d = lint(journal, url);
        diagnostics.append(&mut d);
    };
    (journal, diagnostics)
}
