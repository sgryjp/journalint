use std::collections::HashMap;
use std::fs::write;
use std::path::Path;

use lsp_types::{TextEdit, Url, WorkspaceEdit};

use crate::code::Code;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::service::ServerState;

pub const RECALCULATE_DURATION: &str = "journalint.recalculateDuration";
pub const ALL_COMMANDS: [&str; 1] = [RECALCULATE_DURATION];

pub trait Command {
    fn title(&self) -> &str;
    fn command(&self) -> &str;
    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Option<WorkspaceEdit>;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandParams {
    url: Url,
    range: lsp_types::Range,
}

#[derive(Debug)]
pub struct RecalculateDuration {}

impl Command for RecalculateDuration {
    fn title(&self) -> &str {
        "Recalculate duration by the interval between start and end time"
    }

    fn command(&self) -> &str {
        RECALCULATE_DURATION
    }

    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Option<WorkspaceEdit> {
        // Find a diagnostic at the specified location with appropriate code
        let Some(diagnostic) = state.diagnostics.get(url).and_then(|diagnostic| {
            diagnostic
                .iter()
                .find(|d| d.is_in_lsp_range(range) && *d.code() == Code::IncorrectDuration)
        }) else {
            return None;
        };

        // Create an edit data in the file to fix the issue
        let Some(new_text) = diagnostic.expectation() else {
            return None;
        };
        let edit = TextEdit::new(diagnostic.lsp_range(), new_text.clone());

        // Compose a "workspace edit" from it
        let edits = HashMap::from([(url.clone(), vec![edit])]);
        Some(WorkspaceEdit::new(edits))
    }
}

pub fn list_available_code_actions(code: &Code) -> Option<Vec<Box<dyn Command>>> {
    match code {
        Code::ParseError => None,
        Code::MismatchedDates => None,
        Code::InvalidStartTime => None,
        Code::InvalidEndTime => None,
        Code::MissingDate => None,
        Code::MissingStartTime => None,
        Code::MissingEndTime => None,
        Code::TimeJumped => None,
        Code::NegativeTimeRange => None,
        Code::IncorrectDuration => Some(vec![Box::new(RecalculateDuration {})]),
    }
}

pub fn get_command_by_name(name: &str) -> Option<Box<dyn Command>> {
    match name {
        RECALCULATE_DURATION => Some(Box::new(RecalculateDuration {})),
        _ => None,
    }
}

pub fn fix(diagnostic: &Diagnostic, content: &str, path: &Path) -> Result<(), JournalintError> {
    // TODO: Move somewhere else
    let span = diagnostic.span();
    let (start, end) = (span.start, span.end);

    if let Some(expectation) = diagnostic.expectation() {
        let mut buf = String::with_capacity(content.len());
        buf.push_str(&content[..start]);
        buf.push_str(expectation.as_str());
        buf.push_str(&content[end..]);
        write(path, buf)?;
    };
    Ok(())
}
