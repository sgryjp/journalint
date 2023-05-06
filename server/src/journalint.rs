use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::Parser;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};

use crate::linemap::LineMap;
use crate::parsing::journal::Journal;

pub struct Journalint<'a> {
    content: &'a str,
    diagnostics: Vec<Diagnostic>,
    linemap: LineMap,
    journal: Option<Journal>,
}

impl<'a> Journalint<'a> {
    pub fn new(filename: Option<String>, content: &'a str) -> Self {
        let mut journalint = Self {
            content,
            diagnostics: Vec::new(),
            linemap: LineMap::new(content),
            journal: None,
        };
        journalint._parse(filename, content);
        journalint
    }

    pub fn journal(&self) -> Option<&Journal> {
        self.journal.as_ref()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_ref()
    }

    /// Parse a journal file content.
    fn _parse(&mut self, filename: Option<String>, content: &str) {
        let (journal, errors) = crate::parsing::journal::journal().parse_recovery_verbose(content);
        self.journal = journal;
        for e in errors {
            let start = self.linemap.position_from_offset(e.span().start);
            let end = self.linemap.position_from_offset(e.span().end);
            let diagnostic = Diagnostic::new(
                Range { start, end },
                Some(DiagnosticSeverity::ERROR),
                Some(NumberOrString::Number(900)),
                filename.clone(),
                e.to_string(),
                None,
                None,
            );
            self.diagnostics.push(diagnostic);
        }
    }

    pub fn report(&self) {
        self.diagnostics
            .iter()
            .for_each(|d| _report_diagnostic(self.content, &self.linemap, d))
    }
}

fn _report_diagnostic(content: &str, linemap: &LineMap, diag: &Diagnostic) {
    let filename = diag.source.as_deref().unwrap_or("<STDIN>");
    let start = linemap.offset_from_position(&diag.range.start);
    let end = linemap.offset_from_position(&diag.range.end);
    let message = &diag.message;

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
