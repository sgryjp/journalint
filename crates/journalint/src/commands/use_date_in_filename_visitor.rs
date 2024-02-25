use std::ops::Range;
use std::path::PathBuf;

use chrono::NaiveDate;
use lsp_types::Url;

use crate::ast::{walk, Expr, Visitor};
use crate::errors::JournalintError;
use crate::textedit::TextEdit;

#[derive(Debug, Default)]
struct UseDateInFilenameVisitor {
    fm_date_span: Range<usize>,
}

impl UseDateInFilenameVisitor {
    pub fn fm_date_span(&self) -> &Range<usize> {
        &self.fm_date_span
    }
}

impl Visitor for UseDateInFilenameVisitor {
    fn on_visit_fm_date(
        &mut self,
        _value: &NaiveDate,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.fm_date_span = span.clone();
        Ok(())
    }
}

pub(super) fn execute(url: &Url, ast_root: &Expr) -> Result<Option<TextEdit>, JournalintError> {
    // Determine where to edit.
    let mut visitor = UseDateInFilenameVisitor::default();
    walk(ast_root, &mut visitor)?;
    let range_to_replace = visitor.fm_date_span();

    // Generate the new value.
    let new_value = PathBuf::from(url.path())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| JournalintError::UnsupportedUrl { url: url.clone() })
        .and_then(|stem| NaiveDate::parse_from_str(stem, "%Y-%m-%d").map_err(JournalintError::from))
        .map(|date| date.format("%Y-%m-%d").to_string())?;

    Ok(Some(TextEdit::new(range_to_replace.clone(), new_value)))
}
