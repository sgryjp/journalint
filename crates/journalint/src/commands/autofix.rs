//! Autofix commands
use std::sync::Arc;

use lsp_types::{Url, WorkspaceEdit};
use strum::EnumIter;

use crate::ast::Expr;
use crate::code::Code;
use crate::commands::Command;
use crate::errors::JournalintError;
use crate::linemap::LineMap;

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
    /// * `url` - URL of the document
    /// * `line_map` - Line-column mapper for the document
    /// * `ast` - AST of the document
    /// * `selection_range` - Range of the selection at the time this command was invoked.
    fn execute(
        &self,
        url: &Url,
        line_map: &Arc<LineMap>,
        ast: &Expr,
        selection_range: &lsp_types::Range,
    ) -> Result<Option<WorkspaceEdit>, JournalintError> {
        let selection = line_map.lsp_range_to_span(selection_range);

        match self {
            AutofixCommand::RecalculateDuration => {
                recalculate_duration::execute(url, line_map, ast, &selection)
            }
            AutofixCommand::ReplaceWithPreviousEndTime => {
                replace_with_previous_end_time::execute(url, line_map, ast, &selection)
            }
            AutofixCommand::UseDateInFilename => {
                use_date_in_filename_visitor::execute(url, line_map, ast)
            }
        }
    }
}
