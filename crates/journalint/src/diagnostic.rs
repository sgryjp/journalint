use core::ops::Range;

use lsp_types::DiagnosticSeverity;
use lsp_types::NumberOrString;

use crate::code::Code;
use crate::linemap::LineMap;

static SOURCE_NAME: &str = "journalint";

/// Internal diagnostic data structure.
///
/// This is basically the same as lsp_types::Diagnostic except that this has a field
/// `span` of type Range<usize>, not a field `range` of type lsp_types::Range.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    span: Range<usize>,
    code: Code,
    severity: DiagnosticSeverity,
    message: String,
}

impl Diagnostic {
    pub fn new_warning(span: Range<usize>, code: Code, message: String) -> Self {
        Self {
            span,
            code,
            severity: DiagnosticSeverity::WARNING,
            message,
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

    pub fn to_lsp_types(&self, linemap: &LineMap) -> lsp_types::Diagnostic {
        let code = self.code().to_str().to_string();
        let range = lsp_types::Range::new(
            linemap.position_from_offset(self.span().start),
            linemap.position_from_offset(self.span().end),
        );
        lsp_types::Diagnostic::new(
            range,
            Some(self.severity()),
            Some(NumberOrString::String(code)),
            Some(SOURCE_NAME.to_string()),
            self.message().to_owned(),
            None,
            None,
        )
    }
}
