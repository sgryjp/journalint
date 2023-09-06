use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::code::Code;
use crate::diagnostic::Diagnostic;
use crate::linemap::LineMap;
use crate::lint::lint;

pub struct Journalint {
    diagnostics: Vec<Diagnostic>,
}

impl Journalint {
    fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn report(&self, filename: Option<&str>, content: &str) {
        self.diagnostics
            .iter()
            .for_each(|d| _report_diagnostic(content, filename, d))
    }
}

pub fn parse_and_lint(content: &str, source: Option<&str>) -> crate::journalint::Journalint {
    let line_map = Arc::new(LineMap::new(content));

    // Parse
    let (journal, errors) = crate::parse::parse(content);
    let mut diagnostics = errors
        .iter()
        .map(|e| {
            Diagnostic::new_warning(
                e.span(),
                Code::ParseError,
                format!("Parse error: {}", e),
                None,
                line_map.clone(),
            )
        })
        .collect::<Vec<Diagnostic>>();

    // Lint
    if let Some(journal) = journal {
        diagnostics.append(&mut lint(&journal, source, line_map));
    }

    crate::journalint::Journalint::new(diagnostics)
}

fn _report_diagnostic(content: &str, filename: Option<&str>, diag: &Diagnostic) {
    let stdin_source_name = "<STDIN>".to_string();
    let filename = filename.unwrap_or(&stdin_source_name);
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
