use core::ops::Range;

use chumsky::error::Simple;
use url::Url;

use crate::rule::Rule;

/// Internal diagnostic data structure.
///
/// This is basically the same as `lsp_types::Diagnostic` except that this has a field
/// `span` of type `Range<usize>`, not a field `range` of type `lsp_types::Range`.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    span: Range<usize>,
    rule: Rule,
    message: String,
    related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

impl Diagnostic {
    pub fn new_warning(
        span: Range<usize>,
        rule: Rule,
        message: String,
        related_information: Option<Vec<DiagnosticRelatedInformation>>,
    ) -> Self {
        Self {
            span,
            rule,
            message,
            related_information,
        }
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn related_information(&self) -> Option<&[DiagnosticRelatedInformation]> {
        self.related_information.as_deref()
    }
}

impl From<&Simple<char>> for Diagnostic {
    fn from(value: &Simple<char>) -> Self {
        Diagnostic::new_warning(
            value.span(),
            Rule::ParseError,
            format!("Parse error: {value}"),
            None,
        )
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticRelatedInformation {
    uri: Url,
    range: Range<usize>,
    message: String,
}

impl DiagnosticRelatedInformation {
    pub fn new(uri: Url, range: Range<usize>, message: String) -> Self {
        Self {
            uri,
            range,
            message,
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
