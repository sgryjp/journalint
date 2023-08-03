use std::ops::Range;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use lsp_types::DiagnosticSeverity;

use crate::diagnostic::Diagnostic;
use crate::ng::parser2::{Expr, LooseTime};

// Ruff は Visitor パターンで処理している。
// Visitor パターンで特定種別の Node を visit する仕組みを作っておき、
// Checker struct に Visitor を implement して visit_stmt などをオーバーライドしてチェック。

#[derive(Default)]
pub struct Linter {
    source: Option<String>,
    diagnostics: Vec<Diagnostic>,

    fm_date: Option<NaiveDate>,
    fm_start: Option<(LooseTime, Range<usize>)>,
    fm_start_resolved: Option<DateTime<Utc>>,
    fm_end: Option<(LooseTime, Range<usize>)>,
    fm_end_resolved: Option<DateTime<Utc>>,

    entry_start_value: Option<DateTime<Utc>>,
    entry_end_value: Option<DateTime<Utc>>,
    entry_end_span: Option<Range<usize>>,
}

impl Linter {
    pub fn new(source: Option<String>) -> Linter {
        Linter {
            source,
            ..Default::default()
        }
    }

    fn on_visit_frontmatter_date(&mut self, date: &NaiveDate, _span: &Range<usize>) {
        self.fm_date = Some(*date);
    }

    fn on_visit_frontmatter_starttime(&mut self, start_time: &LooseTime, span: &Range<usize>) {
        // TODO:
        // Rename
        self.fm_start = Some((start_time.clone(), span.clone()));
    }

    fn on_visit_frontmatter_endtime(&mut self, end_time: &LooseTime, span: &Range<usize>) {
        self.fm_end = Some((end_time.clone(), span.clone()));
    }

    fn on_leave_frontmatter(
        &mut self,
        _date: &Expr,
        _start: &Expr,
        _end: &Expr,
        span: &Range<usize>,
    ) {
        // Calculate exact time of start and end
        if let (Some(date), Some((start, start_span))) = (self.fm_date, self.fm_start.as_ref()) {
            self.fm_start_resolved = match start.to_datetime(&date) {
                Ok(dt) => Some(dt),
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new(
                        start_span.clone(),
                        DiagnosticSeverity::WARNING,
                        self.source.clone(),
                        format!("invalid start time: {}", e),
                    ));
                    None
                }
            };
        }
        if let (Some(date), Some((end, end_span))) = (self.fm_date, self.fm_end.as_ref()) {
            self.fm_end_resolved = match end.to_datetime(&date) {
                Ok(dt) => Some(dt),
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new(
                        end_span.clone(),
                        DiagnosticSeverity::WARNING,
                        self.source.clone(),
                        format!("invalid end time: {}", e),
                    ));
                    None
                }
            };
        }

        // Warn if one of date, start and end is missing
        if self.fm_date.is_none() {
            self.diagnostics.push(Diagnostic::new(
                span.clone(),
                DiagnosticSeverity::WARNING,
                self.source.clone(),
                "date field is missing".to_string(),
            ));
        }
        if self.fm_start.is_none() {
            self.diagnostics.push(Diagnostic::new(
                span.clone(),
                DiagnosticSeverity::WARNING,
                self.source.clone(),
                "start field is missing".to_string(),
            ));
        }
        if self.fm_end.is_none() {
            self.diagnostics.push(Diagnostic::new(
                span.clone(),
                DiagnosticSeverity::WARNING,
                self.source.clone(),
                "end field is missing".to_string(),
            ));
        }
    }

    fn on_leave_entry(
        &mut self,
        _start_time: &Expr,
        _end_time: &Expr,
        _codes: &Vec<Expr>,
        _duration: &Expr,
        _span: &Range<usize>,
    ) {
        self.entry_start_value = None;
        self.entry_end_value = None;
        self.entry_end_span = None;
    }

    fn on_visit_start_time(&mut self, start_time: &LooseTime, span: &Range<usize>) {
        if let Some(date) = self.fm_date {
            match start_time.to_datetime(&date) {
                Ok(d) => {
                    self.entry_start_value = Some(d);
                }
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new(
                        span.clone(),
                        DiagnosticSeverity::WARNING,
                        self.source.clone(),
                        e.to_string(),
                    ));
                }
            }
        }
    }

    fn on_visit_end_time(&mut self, end_time: &LooseTime, span: &Range<usize>) {
        if let Some(date) = self.fm_date {
            match end_time.to_datetime(&date) {
                Ok(d) => {
                    self.entry_end_value = Some(d);
                    self.entry_end_span = Some(span.clone());
                }
                Err(e) => {
                    self.diagnostics.push(Diagnostic::new(
                        span.clone(),
                        DiagnosticSeverity::WARNING,
                        self.source.clone(),
                        e.to_string(),
                    ));
                }
            }
        }
    }

    fn on_visit_duration(&mut self, duration: &Duration, span: &Range<usize>) {
        let start = self.entry_start_value.unwrap();
        let end = self.entry_end_value.unwrap();

        let Ok(calculated) = (end - start).to_std() else {
            self.diagnostics.push(Diagnostic::new(
                self.entry_end_span.as_ref().unwrap().clone(),
                DiagnosticSeverity::WARNING,
                self.source.clone(),
                format!(
                    "End time must be ahead of start time: {}-{}",
                    start.format("%H:%M"),
                    end.format("%H:%M")
                ),
            ));
            return;
        };
        let written = duration;
        if calculated != *written {
            self.diagnostics.push(Diagnostic::new(
                span.clone(),
                DiagnosticSeverity::WARNING,
                self.source.clone(),
                format!(
                    "Incorrect duration: found {:1.2}, expected {:1.2}",
                    written.as_secs_f64() / 3600.0,
                    calculated.as_secs_f64() / 3600.0
                ),
            ));
        }
    }
}

fn walk(expr: &Expr, visitor: &mut Linter) {
    match expr {
        Expr::FrontMatterDate { value, span } => {
            visitor.on_visit_frontmatter_date(value, span);
        }
        Expr::FrontMatterStartTime { value, span } => {
            visitor.on_visit_frontmatter_starttime(value, span);
        }
        Expr::FrontMatterEndTime { value, span } => {
            visitor.on_visit_frontmatter_endtime(value, span);
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
            visitor.on_leave_frontmatter(date, start, end, span);
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
            span: _,
        } => {
            walk(start, visitor);
            walk(end, visitor);
            for code in codes {
                walk(code, visitor);
            }
            walk(duration, visitor);
            walk(activity, visitor);
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
