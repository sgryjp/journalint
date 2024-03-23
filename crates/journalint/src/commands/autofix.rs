//! Autofix commands
use std::ops::Range;

use lsp_types::Url;
use strum::EnumIter;

use journalint_parse::ast::Expr;

use crate::commands::Command;
use crate::errors::JournalintError;
use crate::textedit::TextEdit;
use crate::violation::Violation;

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

    fn can_fix(&self, violation: &Violation) -> bool {
        match self {
            AutofixCommand::RecalculateDuration => *violation == Violation::IncorrectDuration,
            AutofixCommand::ReplaceWithPreviousEndTime => *violation == Violation::TimeJumped,
            AutofixCommand::UseDateInFilename => *violation == Violation::MismatchedDates,
        }
    }

    /// Execute an auto-fix command.
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the document
    /// * `ast_root` - AST of the document
    /// * `selection` - Span of the selection at the time this command was invoked.
    fn execute(
        &self,
        url: &Url,
        ast_root: &Expr,
        selection: &Range<usize>,
    ) -> Result<Option<TextEdit>, JournalintError> {
        match self {
            AutofixCommand::RecalculateDuration => {
                recalculate_duration::execute(url, ast_root, selection)
            }
            AutofixCommand::ReplaceWithPreviousEndTime => {
                replace_with_previous_end_time::execute(url, ast_root, selection)
            }
            AutofixCommand::UseDateInFilename => {
                use_date_in_filename_visitor::execute(url, ast_root)
            }
        }
    }
}
