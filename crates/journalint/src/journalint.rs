use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::Parser;

use crate::ast::{Journal, JournalEntry};
use crate::diagnostic::{Diagnostic, DiagnosticSeverity};
use crate::linemap::LineMap;
use crate::parsers;

pub struct Journalint<'a> {
    pub source: Option<String>, // TODO: make these fields private
    pub content: &'a str,
    pub diagnostics: Vec<Diagnostic>,
    pub linemap: LineMap,
}

impl<'a> Journalint<'a> {
    pub fn new(filename: Option<String>, content: &'a str) -> Self {
        let (journal, mut diagnostics) = Journalint::parse(filename.clone(), content);
        if let Some(journal) = journal {
            diagnostics.append(&mut Journalint::lint(&journal, filename.clone()));
        }

        Self {
            source: filename,
            content,
            diagnostics,
            linemap: LineMap::new(content),
        }
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
    fn parse(filename: Option<String>, content: &str) -> (Option<Journal>, Vec<Diagnostic>) {
        let (journal, errors) = parsers::journal().parse_recovery(content);
        (
            journal,
            errors
                .iter()
                .map(|e| {
                    Diagnostic::new(
                        e.span(),
                        DiagnosticSeverity::ERROR,
                        filename.clone(),
                        e.to_string(),
                    )
                })
                .collect(),
        )
    }

    fn lint(journal: &Journal, source: Option<String>) -> Vec<Diagnostic> {
        // Scan entries
        journal
            .entries()
            .iter()
            .filter_map(|entry| {
                let start_time = entry.time_range().start();
                let Some(start) = start_time.to_datetime(journal.front_matter().date()) else {
                    return Some(Diagnostic::new(
                        start_time.span().clone(),
                        DiagnosticSeverity::WARNING,
                        source.clone(),
                        "invalid start time (out of valid range)".to_string(),
                    ));
                };

                let end_time = entry.time_range().end();
                let Some(end) = end_time.to_datetime(journal.front_matter().date()) else {
                    return Some(Diagnostic::new(
                        end_time.span().clone(),
                        DiagnosticSeverity::WARNING,
                        source.clone(),
                        "invalid end time (out of valid range)".to_string(),
                    ));
                };
                let Ok(calculated_duration) = (end - start).to_std() else {
                    return Some(Diagnostic::new(
                        end_time.span().clone(),
                        DiagnosticSeverity::WARNING,
                        source.clone(),
                        "end time should be the same or after the start time".to_string(),
                    ));
                };

                Self::lint_incorrect_duration(source.as_deref(), calculated_duration, entry)
            })
            .collect()
    }

    pub fn report(&self) {
        self.diagnostics
            .iter()
            .for_each(|d| _report_diagnostic(self.content, d))
    }

    fn lint_incorrect_duration(
        source: Option<&str>,
        calculated_duration: std::time::Duration,
        entry: &JournalEntry,
    ) -> Option<Diagnostic> {
        (&calculated_duration != entry.duration().value()).then(|| {
            let written_duration = entry.duration().value().as_secs_f64() / 3600.0;
            let expected = calculated_duration.as_secs_f64() / 3600.0;
            Diagnostic::new(
                entry.duration().span().clone(),
                DiagnosticSeverity::WARNING,
                source.map(|s| s.to_string()),
                format!(
                    "Incorrect duration: found {:1.2}, expected {:1.2}",
                    written_duration, expected
                ),
            )
        })
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

#[cfg(test)]
mod tests {
    use crate::journalint::Journalint;

    #[test]
    fn incorrect_duration() {
        const TEST_DATA: &str = "\
        ---\n\
        date: 2006-01-02\n\
        start: 15:04\n\
        end: 17:29\n\
        ---\n\
        \n\
        - 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz\n\
        ";

        let journalint = Journalint::new(None, TEST_DATA);
        let diagnostics = journalint.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(*diagnostic.span(), 77..81);
        assert_eq!(
            diagnostic.message(),
            "Incorrect duration: found 1.00, expected 1.25"
        );
    }
}
