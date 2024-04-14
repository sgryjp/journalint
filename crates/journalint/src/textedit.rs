use std::ops::Range;
use std::sync::Arc;

use crate::linemap::LineMap;

/// Represents a text replacement operation.
#[derive(Debug)]
pub struct TextEdit {
    /// Where to replace.
    span: Range<usize>,

    /// The string to be inserted.
    new_text: String,
}

impl TextEdit {
    pub(super) fn new(span: Range<usize>, new_text: String) -> Self {
        Self { span, new_text }
    }

    pub(super) fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub(super) fn new_text(&self) -> &str {
        &self.new_text
    }

    pub(super) fn to_lsp_type(&self, line_map: &Arc<LineMap>) -> lsp_types::TextEdit {
        let range = line_map.span_to_lsp_range(self.span());
        let new_text = self.new_text().to_owned();
        lsp_types::TextEdit::new(range, new_text)
    }

    /// Apply the edit to the given text.
    pub fn apply(&self, content: &mut String) {
        content.replace_range(self.span().clone(), self.new_text());
    }
}
