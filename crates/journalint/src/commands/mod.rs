//! This module provides commands of journalint language server.
mod autofix;
mod recalculate_duration;
mod replace_with_previous_end_time;
mod use_date_in_filename_visitor;

use std::fs::write;
use std::ops::Range;

use log::warn;
use lsp_types::Url;

use crate::ast::Expr;
use crate::code::Code;
pub use crate::commands::autofix::AutofixCommand;
use crate::errors::JournalintError;
use crate::textedit::TextEdit;

/// Command of journalint.
///
/// Currently I only think of auto-fix commands so this must be unsuitable for other kinds of
/// commands.
pub trait Command {
    /// Get short description of this command which is meant to be used in UI.
    fn title(&self) -> &str;

    /// Get machine-readable identifier of this command.
    fn id(&self) -> &str;

    /// Check whether the specified diagnostic code can be fixed by this command or not.
    fn can_fix(&self, code: &Code) -> bool;

    /// Executes this command.
    ///
    /// In case of fix commands, the result is the change set to be applied to the document.
    /// Note that it will be `Ok(None)` if there is nothing to do.
    fn execute(
        &self,
        url: &Url,
        ast_root: &Expr,
        selection: &Range<usize>,
    ) -> Result<Option<TextEdit>, JournalintError>;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandParams {
    url: Url,
    range: lsp_types::Range,
}

// -----------------------------------------------------------------------------

/// Apply a text edit to a local file.
pub(crate) fn apply_text_edit(url: &Url, text_edit: TextEdit) -> Result<(), JournalintError> {
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
    buf.push_str(&content[..text_edit.span().start]);
    buf.push_str(text_edit.new_text());
    buf.push_str(&content[text_edit.span().end..]);

    // Write the partly replaced content back.
    write(&path, buf)?;

    Ok(())
}
