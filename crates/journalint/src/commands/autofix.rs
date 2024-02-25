//! Autofix commands
use std::collections::HashMap;
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

    fn can_fix(&self, code: &Code) -> bool {
        match self {
            AutofixCommand::RecalculateDuration => *code == Code::IncorrectDuration,
            AutofixCommand::ReplaceWithPreviousEndTime => *code == Code::TimeJumped,
            AutofixCommand::UseDateInFilename => *code == Code::MismatchedDates,
        }
    }

    /// Execute an auto-fix command.
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the document
    /// * `line_map` - Line-column mapper for the document
    /// * `ast_root` - AST of the document
    /// * `selection_range` - Range of the selection at the time this command was invoked.
    fn execute(
        &self,
        url: &Url,
        line_map: &Arc<LineMap>,
        ast_root: &Expr,
        selection_range: &lsp_types::Range,
    ) -> Result<Option<WorkspaceEdit>, JournalintError> {
        let selection = line_map.lsp_range_to_span(selection_range);

        let edit = match self {
            AutofixCommand::RecalculateDuration => {
                recalculate_duration::execute(url, ast_root, &selection)
            }
            AutofixCommand::ReplaceWithPreviousEndTime => {
                replace_with_previous_end_time::execute(url, ast_root, &selection)
            }
            AutofixCommand::UseDateInFilename => {
                use_date_in_filename_visitor::execute(url, ast_root)
            }
        }?;
        let Some(edit) = edit else {
            return Ok(None);
        };

        let range = line_map.span_to_lsp_range(edit.span());
        let edit = lsp_types::TextEdit::new(range, edit.new_text().to_string());
        let edits = HashMap::from([(url.clone(), vec![edit])]);
        Ok(Some(WorkspaceEdit::new(edits)))
    }
}
