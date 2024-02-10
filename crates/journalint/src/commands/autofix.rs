//! Autofix commands
use std::collections::HashMap;

use lsp_types::{TextEdit, Url, WorkspaceEdit};
use strum::EnumIter;

use crate::code::Code;
use crate::commands::Command;
use crate::errors::JournalintError;
use crate::service::ServerState;

/// Auto-fix command.
#[derive(Debug, EnumIter)]
pub enum AutofixCommand {
    RecalculateDuration,
    ReplaceWithPreviousEndTime,
    UseDateInFilename,
}

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
