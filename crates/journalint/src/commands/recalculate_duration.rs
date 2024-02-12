use std::cmp::{max, min};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use chrono::prelude::NaiveDate;
use lsp_types::{TextEdit, Url, WorkspaceEdit};

use crate::{
    ast::{walk, Expr, LooseTime, Visitor},
    errors::JournalintError,
    linemap::LineMap,
};

#[derive(Debug, Default)]
struct RecalculateDurationVisitor {
    target_span: Range<usize>,

    fm_date_value: Option<NaiveDate>,
    start_time_value: Option<LooseTime>,
    end_time_value: Option<LooseTime>,
    target_duration_span: Option<Range<usize>>,
}

impl RecalculateDurationVisitor {
    fn new(target_span: &Range<usize>) -> Self {
        Self {
            target_span: target_span.clone(),
            ..Default::default()
        }
    }
}

impl Visitor for RecalculateDurationVisitor {
    fn on_visit_fm_date(
        &mut self,
        value: &NaiveDate,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        self.fm_date_value = Some(*value);
        Ok(())
    }

    fn on_visit_start_time(
        &mut self,
        value: &LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if self.target_duration_span.is_none() {
            self.start_time_value = Some(value.clone());
        }
        Ok(())
    }

    fn on_visit_end_time(
        &mut self,
        value: &LooseTime,
        _span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        if self.target_duration_span.is_none() {
            self.end_time_value = Some(value.clone());
        }
        Ok(())
    }

    fn on_visit_duration(
        &mut self,
        _value: &std::time::Duration,
        span: &Range<usize>,
    ) -> Result<(), JournalintError> {
        let start = max(self.target_span.start, span.start);
        let end = min(self.target_span.end, span.end);
        if start <= end {
            self.target_duration_span = Some(span.clone());
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
    let mut visitor = RecalculateDurationVisitor::new(target_span);
    walk(ast, &mut visitor)?;
    let span_to_replace =
        visitor
            .target_duration_span
            .ok_or_else(|| JournalintError::CommandTargetNotFound {
                command: "recalculateDuration".to_string(),
            })?;

    // Generate the new value.
    let date = visitor
        .fm_date_value
        .ok_or(JournalintError::MissingRequiredValue {
            name: "date".to_string(),
        })?;
    let start_time = visitor
        .start_time_value
        .ok_or(JournalintError::MissingRequiredValue {
            name: "start_time".to_string(),
        })
        .and_then(|t| t.to_datetime(date))?;
    let end_time = visitor
        .end_time_value
        .ok_or(JournalintError::MissingRequiredValue {
            name: "end_time".to_string(),
        })
        .and_then(|t| t.to_datetime(date))?;
    let new_value = end_time - start_time;

    // Compose a "workspace edit" from it
    let range = line_map.span_to_lsp_range(&span_to_replace);
    let edit = TextEdit::new(range, format!("{:1.2}", new_value.num_hours() as f64));
    let edits = HashMap::from([(url.clone(), vec![edit])]);
    Ok(Some(WorkspaceEdit::new(edits)))
}
