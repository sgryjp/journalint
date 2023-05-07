use lsp_types::DiagnosticSeverity;

use crate::{diagnostic::Diagnostic, linemap::LineMap, parsing::journal::Journal};

pub fn duration_mismatch(
    source: Option<&String>,
    linemap: &LineMap,
    journal: &Journal,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for entry in journal.entries() {
        let Some(start) = entry.time_range().start().into_datetime(journal.front_matter().date()) else {
            return diagnostics;// TODO: Make this case a warning
        };
        let Some(end) = entry.time_range().end().into_datetime(journal.front_matter().date()) else {
            return diagnostics;// TODO: Make this case a warning
        };
        let Ok(calculated_duration) = (end - start).to_std() else {
            return diagnostics; // TODO: Make this case a warning, using the Err variant
        };

        if &calculated_duration != entry.duration().value() {
            let written_duration = entry.duration().value().as_secs_f64() / 3600.0;
            diagnostics.push(Diagnostic::new(
                entry.duration().span().clone(),
                DiagnosticSeverity::WARNING,
                source.cloned(),
                format!("Duration mistmatch: {}", written_duration),
            ));
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use crate::journalint::Journalint;

    #[test]
    fn duration_mismatch() {
        const TEST_DATA: &str = "\
        ---\n\
        date: 2006-01-02\n\
        start: 15:04\n\
        end: 17:29\n\
        ---\n\
        \n\
        - 09:00-10:15 ABCDEFG8 AB3 1.00 foo: bar: baz\n\
        ";

        let _journalint = Journalint::new(None, TEST_DATA);
        todo!();
    }
}
