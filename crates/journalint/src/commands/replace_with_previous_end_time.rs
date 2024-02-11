use std::{
    cmp::{max, min},
    collections::HashMap,
    ops::Range,
    sync::Arc,
};

use lsp_types::{TextEdit, Url, WorkspaceEdit};

use crate::{
    ast::{walk, Expr, LooseTime, Visitor},
    errors::JournalintError,
    linemap::LineMap,
};

use super::{AutofixCommand, Command};

#[derive(Debug, Default)]
struct ReplaceWithPreviousEndTimeVisitor {
    target_span: Range<usize>,

    prev_end_time_value: Option<LooseTime>,
    prev_end_time_span: Option<Range<usize>>,
    found: bool,
}

impl ReplaceWithPreviousEndTimeVisitor {
    fn new(target_span: &Range<usize>) -> Self {
        Self {
            target_span: target_span.clone(),
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
        if !self.found {
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
        let start = max(self.target_span.start, span.start);
        let end = min(self.target_span.end, span.end);
        if start < end {
            self.found = true;
        }
        Ok(())
    }
}

pub(super) fn execute(
    url: &Url,
    line_map: &Arc<LineMap>,
    ast: &Expr,
    target_span: &Range<usize>,
) -> Result<Option<WorkspaceEdit>, JournalintError> {
    // Determine where to edit.
    let mut visitor = ReplaceWithPreviousEndTimeVisitor::new(target_span);
    walk(ast, &mut visitor)?;
    let span_to_replace =
        visitor
            .prev_end_time_span
            .ok_or_else(|| JournalintError::CommandTargetNotFound {
                command: AutofixCommand::ReplaceWithPreviousEndTime.id().to_string(),
            })?;

    // Generate the new value.
    let new_value = visitor
        .prev_end_time_value
        .map(|dt| dt.as_str().to_string())
        .expect("prev_end_time_value was not available but prev_end_time_span was available.");

    // Compose a "workspace edit" from it
    let range = line_map.span_to_lsp_range(&span_to_replace);
    let edit = TextEdit::new(range, new_value);
    let edits = HashMap::from([(url.clone(), vec![edit])]);
    Ok(Some(WorkspaceEdit::new(edits)))
}
