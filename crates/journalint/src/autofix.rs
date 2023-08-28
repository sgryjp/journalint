use std::fs::write;
use std::path::Path;

use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;

pub fn fix(diagnostic: &Diagnostic, content: &str, path: &Path) -> Result<(), JournalintError> {
    let span = diagnostic.span();
    let (start, end) = (span.start, span.end);

    if let Some(expectation) = diagnostic.expectation() {
        let mut buf = String::with_capacity(content.len());
        buf.push_str(&content[..start]);
        buf.push_str(expectation.as_str());
        buf.push_str(&content[end..]);
        write(path, buf)?;
    };
    Ok(())
}
