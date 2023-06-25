use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::Parser;

use crate::diagnostic::{Diagnostic, DiagnosticSeverity};
use crate::linemap::LineMap;
use crate::linting::lint_incorrect_duration;
use crate::parsing::journal::Journal;

pub struct Journalint<'a> {
    source: Option<String>,
    content: &'a str,
    diagnostics: Vec<Diagnostic>,
    linemap: LineMap,
    journal: Option<Journal>,
}

impl<'a> Journalint<'a> {
    pub fn new(filename: Option<String>, content: &'a str) -> Self {
        let mut journalint = Self {
            source: filename.clone(),
            content,
            diagnostics: Vec::new(),
            linemap: LineMap::new(content),
            journal: None,
        };
        journalint._parse(filename, content);
        journalint._lint();
        journalint
    }

    pub fn journal(&self) -> Option<&Journal> {
        self.journal.as_ref()
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn linemap(&self) -> &LineMap {
        &self.linemap
    }

    /// Parse a journal file content.
    fn _parse(&mut self, filename: Option<String>, content: &str) {
        let (journal, errors) = crate::parsing::journal::journal().parse_recovery_verbose(content);
        self.journal = journal;
        for e in errors {
            let diagnostic = Diagnostic::new(
                e.span(),
                DiagnosticSeverity::ERROR,
                filename.clone(),
                e.to_string(),
            );
            self.diagnostics.push(diagnostic);
        }
    }

    fn _lint(&mut self) {
        let journal = self.journal().unwrap();

        // Scan entries
        for entry in journal.entries() {
            let start_time = entry.time_range().start();
            let Some(start) = start_time.to_datetime(journal.front_matter().date()) else {
               let d = Diagnostic::new(
                    start_time.span().clone(),
                    DiagnosticSeverity::WARNING,
                    self.source().map(|s| s.to_string()),
                    "invalid start time (out of valid range)".to_string()
                );
                self.diagnostics.push(d);
                return;
            };

            let end_time = entry.time_range().end();
            let Some(end) = end_time.to_datetime(journal.front_matter().date()) else {
                let d = Diagnostic::new(
                    end_time.span().clone(),
                    DiagnosticSeverity::WARNING,
                    self.source().map(|s| s.to_string()),
                    "invalid end time (out of valid range)".to_string(),
                );
                self.diagnostics.push(d);
                return;
            };
            let Ok(calculated_duration) = (end - start).to_std() else {
                let d = Diagnostic::new(
                    end_time.span().clone(),
                    DiagnosticSeverity::WARNING,
                    self.source().map(|s| s.to_string()),
                    "end time should be the same or after the start time".to_string()
                );
                self.diagnostics.push(d);
                return;
            };

            if let Some(d) = lint_incorrect_duration(self.source(), calculated_duration, entry) {
                self.diagnostics.push(d);
                return;
            }
        }
    }

    pub fn report(&self) {
        self.diagnostics
            .iter()
            .for_each(|d| _report_diagnostic(self.content, d))
    }
}

fn _report_diagnostic(content: &str, diag: &Diagnostic) {
    let stdin_source_name = "<STDIN>".to_string();
    let filename = diag.source().unwrap_or(&stdin_source_name);
    let start = diag.span().start;
    let end = diag.span().end;
    let message = diag.message();

    Report::build(ReportKind::Error, filename, start)
        .with_message(message)
        .with_label(
            Label::new((filename, start..end))
                .with_color(Color::Red)
                .with_message(message),
        )
        .finish()
        .eprint((filename, Source::from(content)))
        .unwrap();
}
