use log::warn;
use lsp_types::Url;
use std::fs::write;
use std::ops::Range;
use std::sync::Arc;

use crate::errors::JournalintError;
use crate::linemap::LineMap;

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

    pub(super) fn to_lsp_type(&self, line_map: &Arc<LineMap>) -> lsp_types::TextEdit {
        let range = line_map.span_to_lsp_range(self.span());
        let new_text = self.new_text().to_owned();
        lsp_types::TextEdit::new(range, new_text)
    }

    /// Apply a text edit to a local file.
    pub fn apply_to_file(&self, url: &Url) -> Result<(), JournalintError> {
        // Skip if the URL points to non-local file.
        if url.scheme() != "file" {
            warn!(
                "Tried to execute a TextEdit for a URL of which scheme is not `file`: {}",
                url.scheme()
            );
            return Ok(());
        }
        let path = url
            .to_file_path()
            .map_err(|_| JournalintError::UnsupportedUrl { url: url.clone() })?;

        // Read the current file content.
        let content = std::fs::read_to_string(&path)?;

        // Replace the target range
        let mut buf = String::with_capacity(content.len());
        buf.push_str(&content[..self.span().start]);
        buf.push_str(self.new_text());
        buf.push_str(&content[self.span().end..]);

        // Write the partly replaced content back.
        write(&path, buf)?;

        Ok(())
    }
}
