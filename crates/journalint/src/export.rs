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
    fn run(fmt: ExportFormat, journal: ast::Expr, writer: &'a mut impl Write) {
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
        ast::walk(&journal, &mut this);
    }
}

impl<'a> ast::Visitor for Exporter<'a> {
    fn on_visit_fm_date(&mut self, value: &NaiveDate, _span: &Range<usize>) {
        self.date = Some(*value);
    }

    fn on_visit_fm_start(&mut self, _value: &ast::LooseTime, _span: &Range<usize>) {}

    fn on_visit_fm_end(&mut self, _value: &ast::LooseTime, _span: &Range<usize>) {}

    fn on_leave_fm(
        &mut self,
        _date: &ast::Expr,
        _start: &ast::Expr,
        _end: &ast::Expr,
        _span: &Range<usize>,
    ) {
    }

    fn on_visit_entry(
        &mut self,
        _start_time: &ast::Expr,
        _end_time: &ast::Expr,
        _codes: &[ast::Expr],
        _duration: &ast::Expr,
        _activity: &ast::Expr,
        _span: &Range<usize>,
    ) {
        self.curr_start_time = None;
        self.curr_end_time = None;
        self.curr_duration = None;
        self.curr_codes.clear();
        self.curr_activity = None;
    }

    fn on_visit_start_time(&mut self, value: &ast::LooseTime, _span: &Range<usize>) {
        self.curr_start_time = self.date.and_then(|d| value.to_datetime(d).ok());
    }

    fn on_visit_end_time(&mut self, value: &ast::LooseTime, _span: &Range<usize>) {
        self.curr_end_time = self.date.and_then(|d| value.to_datetime(d).ok());
    }

    fn on_visit_duration(&mut self, value: &Duration, _span: &Range<usize>) {
        self.curr_duration = Some(*value);
    }

    fn on_visit_code(&mut self, value: &str, _span: &Range<usize>) {
        self.curr_codes.push(String::from(value));
    }

    fn on_visit_activity(&mut self, value: &str, _span: &Range<usize>) {
        self.curr_activity = Some(String::from(value));
    }

    fn on_leave_entry(
        &mut self,
        _start_time: &ast::Expr,
        _end_time: &ast::Expr,
        _codes: &[ast::Expr],
        _duration: &ast::Expr,
        _activity: &ast::Expr,
        _span: &Range<usize>,
    ) {
        let Some(start_time) = self.curr_start_time else {
            return;
        };
        let Some(end_time) = self.curr_end_time else {
            return;
        };
        let Some(duration) = self.curr_duration else {
            return;
        };
        let Some(activity) = self.curr_activity.as_ref() else {
            return;
        };
        let entry = JournalEntry {
            start_time,
            end_time,
            duration: duration.as_secs(),
            codes: self.curr_codes.clone(),
            activity: activity.clone(),
        };
        let bytes = match self.fmt {
            ExportFormat::Json => serde_json::to_vec(&entry)
                .expect("### CREATING VEC[U8] FOR A JOURNAL ENTRY FAILED ###"),
            ExportFormat::Csv => todo!(), // TODO: Implement
        };
        let mut _nbytes_written = self
            .writer
            .write(bytes.as_slice())
            .expect("### WRITING A JOURNAL ENTRY FAILED ###");
        _nbytes_written += self
            .writer
            .write("\n".as_bytes())
            .expect("### WRITING AN EOL CODE FAILED ###");
    }
}

pub fn export(
    fmt: ExportFormat,
    journal: ast::Expr,
    writer: &mut impl Write,
) -> Result<(), JournalintError> {
    Exporter::run(fmt, journal, writer);
    Ok(()) // TODO: Change visitor method signature so that they can report err
}
