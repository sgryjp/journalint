use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::diagnostic::Diagnostic;
use crate::linemap::LineMap;
use crate::lint::lint;

pub struct Journalint<'a> {
    #[allow(dead_code)]
    source: Option<String>,
    content: &'a str,
    diagnostics: Vec<Diagnostic>,
    linemap: LineMap,
}

impl<'a> Journalint<'a> {
    fn new(source: &Option<String>, content: &'a str, diagnostics: Vec<Diagnostic>) -> Self {
        let source = source.clone();
        let linemap = LineMap::new(content);
        Self {
            source,
            content,
            diagnostics,
            linemap,
        }
    }

    #[allow(dead_code)]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn linemap(&self) -> &LineMap {
        &self.linemap
    }

    pub fn report(&self) {
        self.diagnostics
            .iter()
            .for_each(|d| _report_diagnostic(self.content, d))
    }
}

pub fn parse_and_lint(content: &str, source: Option<String>) -> crate::journalint::Journalint {
    // Parse
    let (journal, errors) = crate::parse::parse(content);
    let mut diagnostics = errors
        .iter()
        .map(|e| Diagnostic::new_warning(e.span(), source.clone(), format!("parse error: {}", e)))
        .collect::<Vec<Diagnostic>>();

    // Lint
    if let Some(journal) = journal {
        diagnostics.append(&mut lint(&journal, source.clone()));
    }

    crate::journalint::Journalint::new(&source, content, diagnostics)
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
