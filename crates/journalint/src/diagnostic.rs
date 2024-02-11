use core::ops::Range;
use std::sync::Arc;

use lsp_types::DiagnosticSeverity;
use lsp_types::NumberOrString;
use lsp_types::Url;

use crate::code::Code;
use crate::linemap::LineMap;

static SOURCE_NAME: &str = "journalint";

/// Internal diagnostic data structure.
///
/// This is basically the same as `lsp_types::Diagnostic` except that this has a field
/// `span` of type `Range<usize>`, not a field `range` of type `lsp_types::Range`.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    span: Range<usize>,
    code: Code,
    severity: DiagnosticSeverity,
    message: String,
    expectation: Option<String>,
    related_informations: Option<Vec<DiagnosticRelatedInformation>>,
    line_map: Arc<LineMap>,
}

impl Diagnostic {
    pub fn new_warning(
        span: Range<usize>,
        code: Code,
        message: String,
        expectation: Option<String>,
        related_informations: Option<Vec<DiagnosticRelatedInformation>>,
        line_mapper: Arc<LineMap>,
    ) -> Self {
        let severity = DiagnosticSeverity::WARNING;
        Self {
            span,
            code,
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

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn expectation(&self) -> Option<&String> {
        self.expectation.as_ref()
    }

    // --- helper methods ---

    pub fn is_in_lsp_range(&self, range: &lsp_types::Range) -> bool {
        let start = self.line_map.offset_from_position(range.start);
        let end = self.line_map.offset_from_position(range.end);
        self.span.start <= start && end <= self.span.end
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
            value
                .related_informations
                .map(|v| v.iter().map(|ri| ri.clone().into()).collect()),
            None,
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
}

impl From<DiagnosticRelatedInformation> for lsp_types::DiagnosticRelatedInformation {
    fn from(value: DiagnosticRelatedInformation) -> Self {
        let start = value.line_map.position_from_offset(value.range.start);
        let end = value.line_map.position_from_offset(value.range.end);
        lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri: value.uri,
                range: lsp_types::Range { start, end },
            },
            message: value.message,
        }
    }
}
