use core::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;

use lsp_types::DiagnosticSeverity;
use lsp_types::NumberOrString;
use lsp_types::TextEdit;
use lsp_types::Url;
use lsp_types::WorkspaceEdit;

use crate::code::Code;
use crate::linemap::LineMap;

static SOURCE_NAME: &str = "journalint";

/// Internal diagnostic data structure.
///
/// This is basically the same as lsp_types::Diagnostic except that this has a field
/// `span` of type Range<usize>, not a field `range` of type lsp_types::Range.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    span: Range<usize>,
    code: Code,
    severity: DiagnosticSeverity,
    message: String,
    expectation: Option<String>,
    line_map: Arc<LineMap>,
}

impl Diagnostic {
    pub fn new_warning(
        span: Range<usize>,
        code: Code,
        message: String,
        expectation: Option<String>,
        line_mapper: Arc<LineMap>,
    ) -> Self {
        let severity = DiagnosticSeverity::WARNING;
        Self {
            span,
            code,
            severity,
            message,
            expectation,
            line_map: line_mapper,
        }
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn expectation(&self) -> Option<&String> {
        self.expectation.as_ref()
    }

    /// Try creating a WorkspaceEdit to fix the problem.
    pub fn fix(&self, url: &Url) -> Option<WorkspaceEdit> {
        // Create an edit data in the file to fix the issue
        let Some(new_text) = self.expectation() else {
            return None;
        };
        let edit = TextEdit::new(self.lsp_range(), new_text.clone());

        // Compose a "workspace edit" from it
        let edits = HashMap::from([(url.clone(), vec![edit])]);
        Some(WorkspaceEdit::new(edits))
    }

    // --- helper methods ---

    pub fn lsp_range(&self) -> lsp_types::Range {
        let start = self.line_map.position_from_offset(self.span.start);
        let end = self.line_map.position_from_offset(self.span.end);
        lsp_types::Range::new(start, end)
    }

    pub fn is_in_lsp_range(&self, range: &lsp_types::Range) -> bool {
        let start = self.line_map.offset_from_position(&range.start);
        let end = self.line_map.offset_from_position(&range.end);
        self.span.contains(&start) && self.span.contains(&end)
    }
}

impl From<Diagnostic> for lsp_types::Diagnostic {
    fn from(value: Diagnostic) -> Self {
        let code = value.code().as_str().to_string();
        let range = lsp_types::Range::new(
            value.line_map.position_from_offset(value.span().start),
            value.line_map.position_from_offset(value.span().end),
        );
        lsp_types::Diagnostic::new(
            range,
            Some(value.severity()),
            Some(NumberOrString::String(code)),
            Some(SOURCE_NAME.to_string()),
            value.message().to_owned(),
            None,
            None,
        )
    }
}
