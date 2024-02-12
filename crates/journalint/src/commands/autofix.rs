//! Autofix commands
use lsp_types::{Url, WorkspaceEdit};
use strum::EnumIter;

use crate::code::Code;
use crate::commands::Command;
use crate::errors::JournalintError;
use crate::service::ServerState;

use super::{recalculate_duration, replace_with_previous_end_time, use_date_in_filename_visitor};

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

    /// Execute an auto-fix command.
    ///
    /// # Arguments
    ///
    /// * `state` - State of the language server
    /// * `url` - URL of the document
    /// * `range` - Range of the selection at the time this command was invoked.
    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Result<Option<WorkspaceEdit>, JournalintError> {
        // Get state of the document
        let doc_state = state.document_state(url)?;
        let line_map = doc_state.line_map();
        let ast = doc_state.ast().ok_or_else(|| {
            JournalintError::UnexpectedError(format!("No AST available for the document: {url}"))
        })?;
        let target_span = line_map.lsp_range_to_span(range);

        match self {
            AutofixCommand::RecalculateDuration => {
                recalculate_duration::execute(url, &line_map, ast, &target_span)
            }
            AutofixCommand::ReplaceWithPreviousEndTime => {
                replace_with_previous_end_time::execute(url, &line_map, ast, &target_span)
            }
            AutofixCommand::UseDateInFilename => {
                use_date_in_filename_visitor::execute(url, &line_map, ast)
            }
        }
    }
}
