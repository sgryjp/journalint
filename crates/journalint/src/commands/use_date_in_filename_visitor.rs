use std::{collections::HashMap, ops::Range, path::PathBuf, sync::Arc};

use chrono::NaiveDate;
use lsp_types::{TextEdit, Url, WorkspaceEdit};

use crate::{
    ast::{walk, Expr, Visitor},
    errors::JournalintError,
    linemap::LineMap,
};

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

pub(super) fn execute(
    url: &Url,
    line_map: &Arc<LineMap>,
    ast: &Expr,
) -> Result<Option<WorkspaceEdit>, JournalintError> {
    // Determine where to edit.
    let mut visitor = UseDateInFilenameVisitor::default();
    walk(ast, &mut visitor)?;
    let range_to_replace = line_map.span_to_lsp_range(visitor.fm_date_span());

    // Generate the new value.
    let new_value = PathBuf::from(url.path())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| JournalintError::UnsupportedUrl { url: url.clone() })
        .and_then(|stem| NaiveDate::parse_from_str(stem, "%Y-%m-%d").map_err(JournalintError::from))
        .map(|date| date.format("%Y-%m-%d").to_string())?;

    // Compose a "workspace edit" from it
    let edit = TextEdit::new(range_to_replace, new_value);
    let edits = HashMap::from([(url.clone(), vec![edit])]);
    Ok(Some(WorkspaceEdit::new(edits)))
}
