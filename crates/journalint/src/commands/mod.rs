//! This module provides commands of journalint language server.
mod autofix;
mod recalculate_duration;
mod replace_with_previous_end_time;
mod use_date_in_filename_visitor;

use std::fs::write;
use std::path::Path;

use lsp_types::{Url, WorkspaceEdit};

use crate::code::Code;
pub use crate::commands::autofix::AutofixCommand;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::service::ServerState;

/// Command of journalint.
///
/// Currently I only think of auto-fix commands so this must be unsuitable for other kinds of
/// commands.
pub trait Command {
    /// Get short description of this command which is meant to be used in UI.
    fn title(&self) -> &str;

    /// Get machine-readable identifier of this command.
    fn id(&self) -> &str;

    /// Get a diagnostic code which is fixable by this command.
    fn fixable_codes(&self) -> Code;

    /// Executes this command.
    ///
    /// In case of fix commands, the result is the change set to be applied to the document.
    /// Note that it will be `Ok(None)` if there is nothing to do.
    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Result<Option<WorkspaceEdit>, JournalintError>;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandParams {
    url: Url,
    range: lsp_types::Range,
}

// -----------------------------------------------------------------------------

pub fn fix(diagnostic: &Diagnostic, content: &str, path: &Path) -> Result<(), JournalintError> {
    // TODO: Move somewhere else
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
