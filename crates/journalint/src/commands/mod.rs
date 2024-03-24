//! This module provides commands of journalint language server.
mod autofix;
mod recalculate_duration;
mod replace_with_previous_end_time;
mod use_date_in_filename_visitor;

use std::ops::Range;

use lsp_types::Url;

use journalint_parse::ast::Expr;
use journalint_parse::violation::Violation;

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

    /// Check whether the specified violation can be fixed by this command or not.
    fn can_fix(&self, violation: &Violation) -> bool;

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
