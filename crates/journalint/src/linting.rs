use lsp_types::DiagnosticSeverity;

use crate::ast;
use crate::diagnostic::Diagnostic;

pub fn lint_incorrect_duration(
    source: Option<&str>,
    calculated_duration: std::time::Duration,
    entry: &ast::JournalEntry,
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
