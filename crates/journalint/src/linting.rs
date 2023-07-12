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
        let d = Diagnostic::new(
            entry.duration().span().clone(),
            DiagnosticSeverity::WARNING,
            source.map(|s| s.to_string()),
            format!(
                "Incorrect duration: found {:1.2}, expected {:1.2}",
                written_duration, expected
            ),
        );
        d
    })
}

pub fn incorrect_duration(source: Option<&str>, journal: &ast::Journal) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for entry in journal.entries() {
        let Some(start) = entry.time_range().start().to_datetime(journal.front_matter().date()) else {
            return diagnostics; // TODO: Make this case a warning
        };
        let Some(end) = entry.time_range().end().to_datetime(journal.front_matter().date()) else {
            return diagnostics; // TODO: Make this case a warning
        };
        let Ok(calculated_duration) = (end - start).to_std() else {
            return diagnostics; // TODO: Make this case a warning, using the Err variant
        };

        if &calculated_duration != entry.duration().value() {
            let written_duration = entry.duration().value().as_secs_f64() / 3600.0;
            let expected = calculated_duration.as_secs_f64() / 3600.0;
            diagnostics.push(Diagnostic::new(
                entry.duration().span().clone(),
                DiagnosticSeverity::WARNING,
                source.map(|s| s.to_owned()),
                format!(
                    "Incorrect duration: found {:1.2}, expected {:1.2}",
                    written_duration, expected
                ),
            ));
        }
    }

    diagnostics
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
