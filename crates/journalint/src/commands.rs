use std::collections::HashMap;
use std::fs::write;
use std::path::Path;

use lsp_types::{TextEdit, Url, WorkspaceEdit};
use once_cell::sync::Lazy;

use crate::code::Code;
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

/// Auto-fix command.
pub enum AutofixCommand {
    RecalculateDuration,
    ReplaceWithPreviousEndTime,
    UseDateInFilename,
}

/// A global static array of all auto-fix commands.
pub static ALL_AUTOFIX_COMMANDS: Lazy<Vec<AutofixCommand>> = Lazy::new(|| {
    vec![
        AutofixCommand::RecalculateDuration,
        AutofixCommand::ReplaceWithPreviousEndTime,
        AutofixCommand::UseDateInFilename,
    ]
});

impl Command for AutofixCommand {
    fn title(&self) -> &str {
        match self {
            AutofixCommand::RecalculateDuration => {
                "Recalculate duration by the interval between start and end time"
            }
            AutofixCommand::ReplaceWithPreviousEndTime => {
                "Replace with the previous entry's end time"
            }
            AutofixCommand::UseDateInFilename => "Use date embedded in the filename",
        }
    }

    fn id(&self) -> &str {
        match self {
            AutofixCommand::RecalculateDuration => "journalint.recalculateDuration",
            AutofixCommand::ReplaceWithPreviousEndTime => "journalint.replaceWithPreviousEndTime",
            AutofixCommand::UseDateInFilename => "journalint.useDateInFilename",
        }
    }

    fn fixable_codes(&self) -> Code {
        match self {
            AutofixCommand::RecalculateDuration => Code::IncorrectDuration,
            AutofixCommand::ReplaceWithPreviousEndTime => Code::TimeJumped,
            AutofixCommand::UseDateInFilename => Code::MismatchedDates,
        }
    }

    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Result<Option<WorkspaceEdit>, JournalintError> {
        execute_fix(self, state, url, range)
    }
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

fn execute_fix(
    command: &dyn Command,
    state: &ServerState,
    url: &Url,
    range: &lsp_types::Range,
) -> Result<Option<WorkspaceEdit>, JournalintError> {
    // Find matching diagnostic object
    let code = command.fixable_codes();
    let diagnostic = state
    .find_diagnostic(url, range, &code)
    .ok_or_else(|| {
        JournalintError::UnexpectedError(format!(
            "No corresponding diagnostic found to fix: {{command: {}, url: {url}, range: {range:?}, code: {code}}}",
            command.id()
        ))
    })?;

    // Create an edit data in the file to fix the issue
    let Some(new_text) = diagnostic.expectation() else {
        return Ok(None);
    };
    let edit = TextEdit::new(diagnostic.lsp_range(), new_text.clone());

    // Compose a "workspace edit" from it
    let edits = HashMap::from([(url.clone(), vec![edit])]);
    Ok(Some(WorkspaceEdit::new(edits)))
}
