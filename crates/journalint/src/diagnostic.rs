use core::ops::Range;
use std::sync::Arc;

use chumsky::error::Simple;
use lsp_types::DiagnosticSeverity;
use lsp_types::Url;

use journalint_parse::violation::Violation;

use crate::linemap::LineMap;

/// Internal diagnostic data structure.
///
/// This is basically the same as `lsp_types::Diagnostic` except that this has a field
/// `span` of type `Range<usize>`, not a field `range` of type `lsp_types::Range`.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    span: Range<usize>,
    violation: Violation,
    severity: DiagnosticSeverity,
    message: String,
    expectation: Option<String>,
    related_informations: Option<Vec<DiagnosticRelatedInformation>>,
    line_map: Arc<LineMap>,
}

impl Diagnostic {
    pub fn new_warning(
        span: Range<usize>,
        violation: Violation,
        message: String,
        expectation: Option<String>,
        related_informations: Option<Vec<DiagnosticRelatedInformation>>,
        line_mapper: Arc<LineMap>,
    ) -> Self {
        let severity = DiagnosticSeverity::WARNING;
        Self {
            span,
            violation,
            severity,
            message,
            expectation,
            related_informations,
            line_map: line_mapper,
        }
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub fn violation(&self) -> &Violation {
        &self.violation
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn expectation(&self) -> Option<&String> {
        self.expectation.as_ref()
    }

    pub fn related_informations(&self) -> Option<&[DiagnosticRelatedInformation]> {
        self.related_informations.as_ref().map(|v| v.as_slice())
    }

    // --- helper methods ---

    pub fn is_in_lsp_range(&self, range: &lsp_types::Range) -> bool {
        let start = self.line_map.offset_from_position(range.start);
        let end = self.line_map.offset_from_position(range.end);
        self.span.start <= start && end <= self.span.end
    }

    pub fn from_parse_error(e: &Simple<char>, line_map: Arc<LineMap>) -> Diagnostic {
        Diagnostic::new_warning(
            e.span(),
            Violation::ParseError,
            format!("Parse error: {e}"),
            None,
            None,
            line_map,
        )
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticRelatedInformation {
    uri: Url,
    range: Range<usize>,
    message: String,
    line_map: Arc<LineMap>,
}

impl DiagnosticRelatedInformation {
    pub fn new(uri: Url, range: Range<usize>, message: String, line_map: Arc<LineMap>) -> Self {
        Self {
            uri,
            range,
            message,
            line_map,
        }
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
