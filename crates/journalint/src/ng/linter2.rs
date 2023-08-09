use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};

use crate::diagnostic::Diagnostic;
use crate::ng::parser2::{Expr, LooseTime};

#[derive(Default)]
pub struct Linter {
    source: Option<String>,
    diagnostics: Vec<Diagnostic>,

    fm_date: Option<(NaiveDate, Range<usize>)>,
    fm_start: Option<(LooseTime, Range<usize>)>,
    fm_start_datetime: Option<DateTime<Utc>>,
    fm_end: Option<(LooseTime, Range<usize>)>,
    fm_end_datetime: Option<DateTime<Utc>>,

    entry_start: Option<(DateTime<Utc>, Range<usize>)>,
    entry_end: Option<(DateTime<Utc>, Range<usize>)>,
    prev_entry_end: Option<(DateTime<Utc>, Range<usize>)>,
}

impl Linter {
    pub fn new(source: Option<String>) -> Linter {
        Linter {
            source,
            ..Default::default()
        }
    }

    fn on_visit_fm_date(&mut self, date: &NaiveDate, span: &Range<usize>) {
        self.fm_date = Some((*date, span.clone()));

        // Check the date value matches the one in the file name
        if let Some(source) = &self.source {
            let source = PathBuf::from(source);
            let Some(date_in_filename) = source.file_stem() else {
                return;
            };
            let Some(date_in_filename) = date_in_filename.to_str() else {
                return;
            };
            if let Ok(date_in_filename) = NaiveDate::parse_from_str(date_in_filename, "%Y-%m-%d") {
                if date_in_filename != *date {
                    self.diagnostics.push(Diagnostic::new_warning(
                        span.clone(),
                        self.source.clone(),
                        format!(
                            "date is different from the one in the filename: expected to be {}",
                            date_in_filename.format("%Y-%m-%d")
                        ),
                    ));
                }
            }
        }
    }

    fn on_visit_fm_start(&mut self, start_time: &LooseTime, span: &Range<usize>) {
        self.fm_start = Some((start_time.clone(), span.clone()));
    }

    fn on_visit_fm_end(&mut self, end_time: &LooseTime, span: &Range<usize>) {
        self.fm_end = Some((end_time.clone(), span.clone()));
    }

    fn on_leave_fm(&mut self, _date: &Expr, _start: &Expr, _end: &Expr, span: &Range<usize>) {
        // Calculate exact time of start and end
        if let (Some((date, _)), Some((start, start_span))) =
            (self.fm_date.as_ref(), self.fm_start.as_ref())
        {
            self.fm_start_datetime = match start.to_datetime(date) {
                Ok(dt) => Some(dt),
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new_warning(
                        start_span.clone(),
                        self.source.clone(),
                        format!("invalid start time: {}", e),
                    ));
                    None
                }
            };
        }
        if let (Some((date, _)), Some((end, end_span))) =
            (self.fm_date.as_ref(), self.fm_end.as_ref())
        {
            self.fm_end_datetime = match end.to_datetime(date) {
                Ok(dt) => Some(dt),
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new_warning(
                        end_span.clone(),
                        self.source.clone(),
                        format!("invalid end time: {}", e),
                    ));
                    None
                }
            };
        }

        // Warn if one of date, start and end is missing
        if self.fm_date.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                self.source.clone(),
                "date field is missing".to_string(),
            ));
        }
        if self.fm_start.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                self.source.clone(),
                "start field is missing".to_string(),
            ));
        }
        if self.fm_end.is_none() {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                self.source.clone(),
                "end field is missing".to_string(),
            ));
        }
    }

    fn on_leave_entry(
        &mut self,
        _start_time: &Expr,
        _end_time: &Expr,
        _codes: &[Expr],
        _duration: &Expr,
        _activity: &Expr,
        _span: &Range<usize>,
    ) {
        self.entry_start = None;
        self.prev_entry_end = self.entry_end.take();
    }

    fn on_visit_start_time(&mut self, start_time: &LooseTime, span: &Range<usize>) {
        if let Some((date, _)) = self.fm_date {
            match start_time.to_datetime(&date) {
                Ok(start_dt) => {
                    self.entry_start = Some((start_dt, span.clone()));

                    // Check if start time matches the end of the previous entry
                    if let Some((prev_end_dt, _)) = self.prev_entry_end {
                        if start_dt != prev_end_dt {
                            self.diagnostics.push(Diagnostic::new_warning(
                                span.clone(),
                                self.source.clone(),
                                format!(
                                    "gap found: previous entry's end time was {}",
                                    prev_end_dt.format("%H:%M")
                                ),
                            ));
                        }
                    }
                }
                Err(e) => {
                    // Start time is not a valid value
                    self.diagnostics.push(Diagnostic::new_warning(
                        span.clone(),
                        self.source.clone(),
                        e.to_string(),
                    ));
                }
            };
        }
    }

    fn on_visit_end_time(&mut self, end_time: &LooseTime, span: &Range<usize>) {
        if let Some((date, _)) = self.fm_date {
            match end_time.to_datetime(&date) {
                Ok(d) => {
                    self.entry_end = Some((d, span.clone()));
                }
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new_warning(
                        span.clone(),
                        self.source.clone(),
                        e.to_string(),
                    ));
                }
            }
        }
    }

    fn on_visit_duration(&mut self, duration: &Duration, span: &Range<usize>) {
        let (start, _) = self.entry_start.as_ref().unwrap();
        let (end, end_span) = self.entry_end.as_ref().unwrap();

        let Ok(calculated) = (*end - *start).to_std() else {
            self.diagnostics.push(Diagnostic::new_warning(
                end_span.clone(),
                self.source.clone(),
                format!(
                    "end time must be ahead of start time: {}-{}",
                    start.format("%H:%M"),
                    end.format("%H:%M")
                ),
            ));
            return;
        };
        let written = duration;
        if calculated != *written {
            self.diagnostics.push(Diagnostic::new_warning(
                span.clone(),
                self.source.clone(),
                format!(
                    "incorrect duration: expected {:1.2}",
                    calculated.as_secs_f64() / 3600.0
                ),
            ));
        }
    }
}

fn walk(expr: &Expr, visitor: &mut Linter) {
    match expr {
        Expr::FrontMatterDate { value, span } => {
            visitor.on_visit_fm_date(value, span);
        }
        Expr::FrontMatterStartTime { value, span } => {
            visitor.on_visit_fm_start(value, span);
        }
        Expr::FrontMatterEndTime { value, span } => {
            visitor.on_visit_fm_end(value, span);
        }
        Expr::FrontMatter {
            date,
            start,
            end,
            span,
        } => {
            walk(date, visitor);
            walk(start, visitor);
            walk(end, visitor);
            visitor.on_leave_fm(date, start, end, span);
        }
        Expr::StartTime { value, span } => {
            visitor.on_visit_start_time(value, span);
        }
        Expr::EndTime { value, span } => {
            visitor.on_visit_end_time(value, span);
        }
        Expr::Duration { value, span } => {
            visitor.on_visit_duration(value, span);
        }
        // Expr::Code { value, span } => todo!(),
        // Expr::Activity { value, span } => todo!(),
        Expr::Entry {
            start,
            end,
            codes,
            duration,
            activity,
            span,
        } => {
            walk(start, visitor);
            walk(end, visitor);
            for code in codes {
                walk(code, visitor);
            }
            walk(duration, visitor);
            walk(activity, visitor);
            visitor.on_leave_entry(start, end, codes, duration, activity, span);
        }
        Expr::Journal {
            front_matter,
            lines,
        } => {
            walk(front_matter, visitor);
            for line in lines {
                walk(line, visitor);
            }
        }
        // Expr::Error { reason, span } => todo!(),
        // Expr::NonTargetLine => todo!(),
        _ => (),
    }
}

pub fn lint(journal: &Expr, source: Option<String>) -> Vec<Diagnostic> {
    let mut visitor = Linter::new(source);
    walk(journal, &mut visitor);
    visitor.diagnostics
}
