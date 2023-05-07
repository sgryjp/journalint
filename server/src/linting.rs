use lsp_types::Diagnostic;

use crate::journalint::Journalint;

pub fn duration_mismatch(journalint: &Journalint) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let Some(journal) = journalint.journal() else {
        return diagnostics;
    };

    for entry in journal.entries() {
        let Some(start) = entry.time_range().start().into_datetime(journal.front_matter().date()) else {
            return diagnostics;
        };
        let Some(end) = entry.time_range().end().into_datetime(journal.front_matter().date()) else {
            return diagnostics;
        };

        // if entry.duration().value() != (end - start).into() {
        //     れんじがめんどくさい
        //     diagnostics.push(Diagnostic::new(
        //         lsp_types::Range::new(Position::new(entry.duration().span()), Position::new()),
        //         format!("Duration mistmatch: {}"),
        //     ));
        // }
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
