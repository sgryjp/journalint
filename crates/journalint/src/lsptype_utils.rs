use std::sync::Arc;

use crate::diagnostic::{Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity};
use crate::linemap::LineMap;

static SOURCE_NAME: &str = "journalint";

pub(crate) trait ToLspDisgnostic {
    fn to_lsptype(&self, line_map: &Arc<LineMap>) -> lsp_types::Diagnostic;
}

impl ToLspDisgnostic for Diagnostic {
    fn to_lsptype(&self, line_map: &Arc<LineMap>) -> lsp_types::Diagnostic {
        let violation = self.violation().as_str().to_string();
        let range = lsp_types::Range::new(
            line_map.position_from_offset(self.span().start),
            line_map.position_from_offset(self.span().end),
        );
        lsp_types::Diagnostic::new(
            range,
            Some(self.severity().to_lsptype()),
            Some(lsp_types::NumberOrString::String(violation)),
            Some(SOURCE_NAME.to_string()),
            self.message().to_owned(),
            self.related_informations()
                .map(|v| v.iter().map(|ri| ri.to_lsptype(line_map)).collect()),
            None,
        )
    }
}

pub(crate) trait ToLspDiagnosticRelatedInformation {
    fn to_lsptype(&self, line_map: &Arc<LineMap>) -> lsp_types::DiagnosticRelatedInformation;
}

impl ToLspDiagnosticRelatedInformation for DiagnosticRelatedInformation {
    fn to_lsptype(&self, line_map: &Arc<LineMap>) -> lsp_types::DiagnosticRelatedInformation {
        let start = line_map.position_from_offset(self.range().start);
        let end = line_map.position_from_offset(self.range().end);
        lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri: self.uri().clone(),
                range: lsp_types::Range { start, end },
            },
            message: self.message().to_string(),
        }
    }
}

pub(crate) trait ToLspDiagnosticSeverity {
    fn to_lsptype(&self) -> lsp_types::DiagnosticSeverity;
}

impl ToLspDiagnosticSeverity for DiagnosticSeverity {
    fn to_lsptype(&self) -> lsp_types::DiagnosticSeverity {
        match self {
            DiagnosticSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => lsp_types::DiagnosticSeverity::INFORMATION,
            DiagnosticSeverity::Hint => lsp_types::DiagnosticSeverity::HINT,
        }
    }
}

impl From<lsp_types::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(value: lsp_types::DiagnosticSeverity) -> Self {
        match value {
            lsp_types::DiagnosticSeverity::ERROR => DiagnosticSeverity::Error,
            lsp_types::DiagnosticSeverity::WARNING => DiagnosticSeverity::Warning,
            lsp_types::DiagnosticSeverity::INFORMATION => DiagnosticSeverity::Information,
            lsp_types::DiagnosticSeverity::HINT => DiagnosticSeverity::Hint,
            _ => {
                log::warn!("Unknown severity value: {:?}", value);
                DiagnosticSeverity::Error
            }
        }
    }
}
