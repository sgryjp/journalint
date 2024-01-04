use std::io::Write;
use std::ops::Range;
use std::time::Duration;

use chrono::prelude::*;

use crate::ast;
use crate::errors::JournalintError;

/// Export data format.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ExportFormat {
    /// JSON Lines.
    Json,

    /// CSV with a header line.
    Csv,
}

#[derive(Debug, serde::Serialize)]
struct JournalEntry {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration: u64, // seconds
    codes: Vec<String>,
    activity: String,
}

struct Exporter<'a> {
    // Initialization parameters
    fmt: ExportFormat,
    writer: &'a mut dyn Write,

    // Object state as a visitor
    date: Option<NaiveDate>,
    curr_start_time: Option<DateTime<Utc>>,
    curr_end_time: Option<DateTime<Utc>>,
    curr_duration: Option<Duration>,
    curr_codes: Vec<String>,
    curr_activity: Option<String>,
}

impl<'a> Exporter<'a> {
    fn run(
        fmt: ExportFormat,
        journal: ast::Expr,
        writer: &'a mut impl Write,
    ) -> Result<(), JournalintError> {
        let mut this = Self {
            fmt,
            writer,
            date: None,
            curr_start_time: None,
            curr_end_time: None,
            curr_duration: None,
            curr_codes: Vec::new(),
            curr_activity: None,
        };
        ast::walk(&journal, &mut this)
    }
}

impl<'a> ast::Visitor for Exporter<'a> {
    fn on_visit_fm_date(
        &mut self,
        value: &NaiveDate,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.date = Some(*value);
        Ok(())
    }

    fn on_visit_fm_start(
        &mut self,
        _value: &ast::LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        Ok(())
    }

    fn on_visit_fm_end(
        &mut self,
        _value: &ast::LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        Ok(())
    }

    fn on_leave_fm(&mut self, _span: &Range<usize>) -> Result<(), JournalintError> {
        Ok(())
    }

    fn on_visit_entry(&mut self, _span: &Range<usize>) -> Result<(), JournalintError> {
        self.curr_start_time = None;
        self.curr_end_time = None;
        self.curr_duration = None;
        self.curr_codes.clear();
        self.curr_activity = None;
        Ok(())
    }

    fn on_visit_start_time(
        &mut self,
        value: &ast::LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.curr_start_time = self.date.and_then(|d| value.to_datetime(d).ok());
        Ok(())
    }

    fn on_visit_end_time(
        &mut self,
        value: &ast::LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.curr_end_time = self.date.and_then(|d| value.to_datetime(d).ok());
        Ok(())
    }

    fn on_visit_duration(
        &mut self,
        value: &Duration,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.curr_duration = Some(*value);
        Ok(())
    }

    fn on_visit_code(&mut self, value: &str, _span: &Range<usize>) -> Result<(), JournalintError> {
        self.curr_codes.push(String::from(value));
        Ok(())
    }

    fn on_visit_activity(
        &mut self,
        value: &str,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.curr_activity = Some(String::from(value));
        Ok(())
    }

    fn on_leave_entry(&mut self, _span: &Range<usize>) -> Result<(), JournalintError> {
        // Skip exporting the entry if any of the components were invalid
        let Some(start_time) = self.curr_start_time else {
            return Ok(());
        };
        let Some(end_time) = self.curr_end_time else {
            return Ok(());
        };
        let Some(duration) = self.curr_duration else {
            return Ok(());
        };
        let Some(activity) = self.curr_activity.as_ref() else {
            return Ok(());
        };

        // Create a struct for serialization purpose
        let entry = JournalEntry {
            start_time,
            end_time,
            duration: duration.as_secs(),
            codes: self.curr_codes.clone(),
            activity: activity.clone(),
        };

        // Serialize
        let bytes = match self.fmt {
            ExportFormat::Json => serde_json::to_vec(&entry).map_err(JournalintError::from)?,
            ExportFormat::Csv => todo!(), // TODO: Implement
        };
        self.writer
            .write_all(bytes.as_slice())
            .map_err(JournalintError::from)?;
        self.writer
            .write_all("\n".as_bytes())
            .map_err(JournalintError::from)?;
        Ok(())
    }
}

pub fn export(
    fmt: ExportFormat,
    journal: ast::Expr,
    writer: &mut impl Write,
) -> Result<(), JournalintError> {
    Exporter::run(fmt, journal, writer)
}
