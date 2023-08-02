mod linter2;
mod parser2;

use lsp_types::DiagnosticSeverity;

use crate::diagnostic::Diagnostic;
use crate::journalint::Journalint;
use crate::ng::parser2::parse;

use self::linter2::lint;

pub fn run(content: &str, source: Option<String>) -> Journalint {
    // Parse
    let (journal, errors) = parse(content);
    let mut diagnostics = errors
        .iter()
        .map(|e| {
            Diagnostic::new(
                e.span(),
                DiagnosticSeverity::WARNING,
                source.clone(),
                format!("{}", e), //TODO:
            )
        })
        .collect::<Vec<Diagnostic>>();

    // Lint
    if let Some(journal) = journal {
        diagnostics.append(&mut lint(&journal, source.clone()));
        //eprintln!("{:?}", journal); //////
    }

    Journalint::new(&source, content, diagnostics)
}
