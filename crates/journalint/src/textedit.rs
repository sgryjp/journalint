use std::ops::Range;

/// Represents a text replacement operation.
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
}
