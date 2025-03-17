use std::sync::Arc;

use journalint_parse::diagnostic::{Diagnostic, DiagnosticRelatedInformation};

use crate::line_mapper::LineMapper;

static SOURCE_NAME: &str = "journalint";

pub(crate) trait ToLspDiagnostic {
    fn to_lsptype(&self, line_mapper: &Arc<LineMapper>) -> lsp_types::Diagnostic;
}

impl ToLspDiagnostic for Diagnostic {
    fn to_lsptype(&self, line_mapper: &Arc<LineMapper>) -> lsp_types::Diagnostic {
        let rule = self.rule().as_str().to_string();
        let range = lsp_types::Range::new(
            line_mapper.position_from_offset(self.span().start),
            line_mapper.position_from_offset(self.span().end),
        );
        lsp_types::Diagnostic::new(
            range,
            Some(lsp_types::DiagnosticSeverity::WARNING),
            Some(lsp_types::NumberOrString::String(rule)),
            Some(SOURCE_NAME.to_string()),
            self.message().to_owned(),
            self.related_information()
                .map(|v| v.iter().map(|ri| ri.to_lsptype(line_mapper)).collect()),
            None,
        )
    }
}

pub(crate) trait ToLspDiagnosticRelatedInformation {
    fn to_lsptype(&self, line_mapper: &Arc<LineMapper>) -> lsp_types::DiagnosticRelatedInformation;
}

impl ToLspDiagnosticRelatedInformation for DiagnosticRelatedInformation {
    fn to_lsptype(&self, line_mapper: &Arc<LineMapper>) -> lsp_types::DiagnosticRelatedInformation {
        let start = line_mapper.position_from_offset(self.range().start);
        let end = line_mapper.position_from_offset(self.range().end);
        lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri: self.uri().clone(),
                range: lsp_types::Range { start, end },
            },
            message: self.message().to_string(),
        }
    }
}
