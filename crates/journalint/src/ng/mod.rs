mod linter2;
mod parser2;

use crate::diagnostic::Diagnostic;
use crate::journalint::Journalint;
use crate::ng::parser2::parse;

use self::linter2::lint;

pub fn run(content: &str, source: Option<String>) -> Journalint {
    // Parse
    let (journal, errors) = parse(content);
    let mut diagnostics = errors
        .iter()
        .map(|e| Diagnostic::new_warning(e.span(), source.clone(), format!("parse error: {}", e)))
        .collect::<Vec<Diagnostic>>();

    // Lint
    if let Some(journal) = journal {
        diagnostics.append(&mut lint(&journal, source.clone()));
    }

    Journalint::new(&source, content, diagnostics)
}
