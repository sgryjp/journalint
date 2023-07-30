use core::ops::Range;

pub use lsp_types::DiagnosticSeverity;

use crate::linemap::LineMap;

/// Internal diagnostic data structure.
///
/// This is basically the same as lsp_types::Diagnostic except that this has a field
/// `span` of type Range<usize>, not a field `range` of type lsp_types::Range.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    span: Range<usize>,
    severity: DiagnosticSeverity,
    source: Option<String>,
    message: String,
}

impl Diagnostic {
    pub fn new(
        // TODO: Defining new_error and new_warning.
        // TODO: Define set_source (builder)
        span: Range<usize>,
        severity: DiagnosticSeverity,
        source: Option<String>,
        message: String,
    ) -> Self {
        Self {
            span,
            severity,
            source,
            message,
        }
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn to_lsp_types(&self, linemap: &LineMap) -> lsp_types::Diagnostic {
        let range = lsp_types::Range::new(
            linemap.position_from_offset(self.span().start),
            linemap.position_from_offset(self.span().end),
        );
        lsp_types::Diagnostic::new(
            range,
            Some(self.severity()),
            None,
            self.source().cloned(),
            self.message().to_owned(),
            None,
            None,
        )
    }
}
