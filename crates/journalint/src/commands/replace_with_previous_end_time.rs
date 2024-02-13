use std::cmp::{max, min};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use lsp_types::{TextEdit, Url, WorkspaceEdit};

use crate::ast::{walk, Expr, LooseTime, Visitor};
use crate::errors::JournalintError;
use crate::linemap::LineMap;

use super::{AutofixCommand, Command};

#[derive(Debug, Default)]
struct ReplaceWithPreviousEndTimeVisitor {
    selection: Range<usize>,

    target_start_time_span: Option<Range<usize>>,
    prev_end_time_value: Option<LooseTime>,
    prev_end_time_span: Option<Range<usize>>,
}

impl ReplaceWithPreviousEndTimeVisitor {
    fn new(selection: Range<usize>) -> Self {
        Self {
            selection,
            ..Default::default()
        }
    }
}

impl Visitor for ReplaceWithPreviousEndTimeVisitor {
    fn on_visit_end_time(
        &mut self,
        value: &crate::ast::LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if self.target_start_time_span.is_none() {
            self.prev_end_time_value = Some(value.clone());
            self.prev_end_time_span = Some(span.clone());
        }
        Ok(())
    }

    fn on_visit_start_time(
        &mut self,
        _value: &crate::ast::LooseTime,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if self.target_start_time_span.is_none() {
            let start = max(self.selection.start, span.start);
            let end = min(self.selection.end, span.end);
            if start <= end {
                self.target_start_time_span = Some(span.clone());
            }
        }
        Ok(())
    }
}

pub(super) fn execute(
    url: &Url,
    line_map: &Arc<LineMap>,
    ast: &Expr,
    selection: &Range<usize>,
) -> Result<Option<WorkspaceEdit>, JournalintError> {
    // Determine where to edit.
    let mut visitor = ReplaceWithPreviousEndTimeVisitor::new(selection.clone());
    walk(ast, &mut visitor)?;
    let span_to_replace = visitor.target_start_time_span.as_ref().ok_or_else(|| {
        JournalintError::CommandTargetNotFound {
            command: AutofixCommand::ReplaceWithPreviousEndTime.id().to_string(),
        }
    })?;

    // Generate the new value.
    let new_value = visitor
        .prev_end_time_value
        .as_ref()
        .map(|dt| dt.as_str().to_string())
        .expect("prev_end_time_value was not available but prev_end_time_span was available.");

    // Compose a "workspace edit" from it
    let range = line_map.span_to_lsp_range(span_to_replace);
    let edit = TextEdit::new(range, new_value);
    let edits = HashMap::from([(url.clone(), vec![edit])]);
    Ok(Some(WorkspaceEdit::new(edits)))
}
