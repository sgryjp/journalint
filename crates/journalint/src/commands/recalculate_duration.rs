use std::cmp::{max, min};
use std::ops::Range;

use chrono::prelude::NaiveDate;
use lsp_types::Url;

use crate::ast::{walk, Expr, LooseTime, Visitor};
use crate::errors::JournalintError;
use crate::textedit::TextEdit;

#[derive(Debug, Default)]
struct RecalculateDurationVisitor {
    selection: Range<usize>,

    fm_date_value: Option<NaiveDate>,
    start_time_value: Option<LooseTime>,
    end_time_value: Option<LooseTime>,
    target_duration_span: Option<Range<usize>>,
}

impl RecalculateDurationVisitor {
    fn new(selection: Range<usize>) -> Self {
        Self {
            selection,
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
        let start = max(self.selection.start, span.start);
        let end = min(self.selection.end, span.end);
        if start <= end {
            self.target_duration_span = Some(span.clone());
        }
        Ok(())
    }
}

pub(super) fn execute(
    _url: &Url,
    ast_root: &Expr,
    selection: &Range<usize>,
) -> Result<Option<TextEdit>, JournalintError> {
    // Determine where to edit.
    let mut visitor = RecalculateDurationVisitor::new(selection.clone());
    walk(ast_root, &mut visitor)?;
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
    let new_value = (new_value.num_seconds() as f64) / 3600.0;

    Ok(Some(TextEdit::new(
        span_to_replace,
        format!("{new_value:1.2}"),
    )))
}
