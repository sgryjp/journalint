//! Provides lint logic.
//!
//! See module `ast` for AST related features, and module `parse` for parsing logic.
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use lsp_types::Url;

use crate::ast::{walk, Expr, LooseTime, Visitor};
use crate::code::Code;
use crate::diagnostic::{Diagnostic, DiagnosticRelatedInformation};
use crate::errors::JournalintError;
use crate::linemap::LineMap;

pub struct Linter<'a> {
    source: &'a Url,
    diagnostics: Vec<Diagnostic>,
    line_map: Arc<LineMap>,

    fm_date: Option<(NaiveDate, Range<usize>)>,
    fm_start: Option<(LooseTime, Range<usize>)>,
    fm_start_datetime: Option<DateTime<Utc>>,
    fm_end: Option<(LooseTime, Range<usize>)>,
    fm_end_datetime: Option<DateTime<Utc>>,

    entry_start: Option<(DateTime<Utc>, Range<usize>)>,
    entry_end: Option<(DateTime<Utc>, Range<usize>)>,
    prev_entry_end: Option<(DateTime<Utc>, Range<usize>)>,
}

impl<'a> Linter<'a> {
    pub fn new(source: &Url, line_map: Arc<LineMap>) -> Linter {
        Linter {
            source,
            diagnostics: vec![],
            line_map,

            fm_date: None,
            fm_start: None,
            fm_start_datetime: None,
            fm_end: None,
            fm_end_datetime: None,

            entry_start: None,
            entry_end: None,
            prev_entry_end: None,
        }
    }

    // Check if date value in the front matter does not match the one in the filename
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
                        Code::MismatchedDates,
                        format!(
                            "Date is different from the one in the filename: expected to be {}",
                            expectation.as_str()
                        ),
                        Some(expectation),
                        None,
                        self.line_map.clone(),
                    ));
                }
            }
        }
    }

    fn check_fm_date_exists(&mut self, span: &Range<usize>) {
        if self.fm_date.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Code::MissingDate,
                "Field 'date' is missing".to_string(),
                None,
                None,
                self.line_map.clone(),
            ));
        }
    }

    fn check_fm_start_exists(&mut self, span: &Range<usize>) {
        if self.fm_start.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Code::MissingStartTime,
                "Field 'start' is missing".to_string(),
                None,
                None,
                self.line_map.clone(),
            ));
        }
    }

    fn check_fm_start_is_valid(&mut self) -> Option<DateTime<Utc>> {
        let Some((date, _)) = self.fm_date.as_ref() else {
            return None;
        };
        let Some((start, start_span)) = self.fm_start.as_ref() else {
            return None;
        };

        match start.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    start_span.clone(),
                    Code::InvalidStartTime,
                    format!("Invalid start time: {e}"),
                    None,
                    None,
                    self.line_map.clone(),
                ));
                None
            }
        }
    }

    fn check_fm_end_exists(&mut self, span: &Range<usize>) {
        if self.fm_end.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                Code::MissingEndTime,
                "Field 'end' is missing".to_string(),
                None,
                None,
                self.line_map.clone(),
            ));
        }
    }

    fn check_fm_end_is_valid(&mut self) -> Option<DateTime<Utc>> {
        let Some((date, _)) = self.fm_date.as_ref() else {
            return None;
        };
        let Some((end, end_span)) = self.fm_end.as_ref() else {
            return None;
        };

        match end.to_datetime(*date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    end_span.clone(),
                    Code::InvalidEndTime,
                    format!("Invalid end time: {e}"),
                    None,
                    None,
                    self.line_map.clone(),
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
                    Code::TimeJumped,
                    format!("The start time does not match the previous entry's end time, which is {expectation}"),
                    Some(expectation),
                    Some(vec![DiagnosticRelatedInformation::new(
                        self.source.clone(),
                        prev_end_range.clone(),
                        format!(
                            "Previous entry's end time is {:02}:{:02}",
                            prev_end_dt.hour(),
                            prev_end_dt.minute()
                        ),
                        self.line_map.clone(),
                    )]),
                    self.line_map.clone(),
                ));
            }
        }
    }

    fn check_start_time(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Option<DateTime<Utc>> {
        let Some((date, _)) = self.fm_date else {
            return None;
        };

        match value.to_datetime(date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                // Start time is not a valid value
                self.diagnostics.push(Diagnostic::new_warning(
                    span.clone(),
                    Code::InvalidStartTime,
                    format!("Invalid start time: {e}"),
                    None,
                    None,
                    self.line_map.clone(),
                ));
                None
            }
        }
    }

    fn check_end_time(&mut self, value: &LooseTime, span: &Range<usize>) -> Option<DateTime<Utc>> {
        let Some((date, _)) = self.fm_date else {
            return None;
        };

        match value.to_datetime(date) {
            Ok(dt) => Some(dt),
            Err(e) => {
                self.diagnostics.push(Diagnostic::new_warning(
                    span.clone(),
                    Code::InvalidEndTime,
                    format!("Invalid end time: {e}"),
                    None,
                    None,
                    self.line_map.clone(),
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
                Code::NegativeTimeRange,
                format!(
                    "End time is not ahead of start time ({})",
                    start.format("%H:%M"),
                ),
                None,
                None,
                self.line_map.clone(),
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
                Code::IncorrectDuration,
                format!("Incorrect duration: expected {expectation}"),
                Some(expectation),
                None,
                self.line_map.clone(),
            ));
        }
    }
}

impl Visitor for Linter<'_> {
    fn on_visit_fm_date(
        &mut self,
        value: &NaiveDate,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.fm_date = Some((*value, span.clone()));
        self.check_fm_date_matches_filename(value, span);
        Ok(())
    }

    fn on_visit_fm_start(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.fm_start = Some((value.clone(), span.clone()));
        Ok(())
    }

    fn on_visit_fm_end(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.fm_end = Some((value.clone(), span.clone()));
        Ok(())
    }

    fn on_leave_fm(&mut self, span: &Range<usize>) -> Result<(), JournalintError> {
        // Calculate exact time of start and end
        self.fm_start_datetime = self.check_fm_start_is_valid();
        self.fm_end_datetime = self.check_fm_end_is_valid();

        // Warn if one of date, start and end is missing
        self.check_fm_date_exists(span);
        self.check_fm_start_exists(span);
        self.check_fm_end_exists(span);

        Ok(())
    }

    fn on_visit_start_time(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if let Some(start_dt) = self.check_start_time(value, span) {
            self.entry_start = Some((start_dt, span.clone()));
            self.check_prev_end_equals_next_start(start_dt, span);
        }
        Ok(())
    }

    fn on_visit_end_time(
        &mut self,
        value: &LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if let Some(dt) = self.check_end_time(value, span) {
            self.entry_end = Some((dt, span.clone()));
        }
        Ok(())
    }

    fn on_visit_duration(
        &mut self,
        value: &Duration,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.check_end_time_exceeds_start_time();
        self.check_duration_matches_end_minus_start(value, span);
        Ok(())
    }
}

pub fn lint(
    journal: &Expr,
    url: &Url,
    line_map: Arc<LineMap>,
) -> Result<Vec<Diagnostic>, JournalintError> {
    let mut visitor = Linter::new(url, line_map);
    walk(journal, &mut visitor)?;
    Ok(visitor.diagnostics)
}
