use std::fs::write;
use std::path::Path;

use lsp_types::{Url, WorkspaceEdit};

use crate::code::Code;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::service::ServerState;

pub const RECALCULATE_DURATION: &str = "journalint.recalculateDuration";
pub const REPLACE_WITH_PREVIOUS_END_TIME: &str = "journalint.replaceWithPreviousEndTime";
pub const ALL_COMMANDS: [&str; 2] = [RECALCULATE_DURATION, REPLACE_WITH_PREVIOUS_END_TIME];

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

// -----------------------------------------------------------------------------
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
        state
            .find_diagnostic(url, range, Code::IncorrectDuration)
            .and_then(|d| d.fix(url))
    }
}

// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ReplaceWithPreviousEndTime {}

impl Command for ReplaceWithPreviousEndTime {
    fn title(&self) -> &str {
        "Replace with the previous entry's end time"
    }

    fn command(&self) -> &str {
        REPLACE_WITH_PREVIOUS_END_TIME
    }

    fn execute(
        &self,
        state: &ServerState,
        url: &Url,
        range: &lsp_types::Range,
    ) -> Option<WorkspaceEdit> {
        state
            .find_diagnostic(url, range, Code::TimeJumped)
            .and_then(|d| d.fix(url))
    }
}

// -----------------------------------------------------------------------------
pub fn list_available_code_actions(code: &Code) -> Option<Vec<Box<dyn Command>>> {
    match code {
        Code::ParseError => None,
        Code::MismatchedDates => None,
        Code::InvalidStartTime => None,
        Code::InvalidEndTime => None,
        Code::MissingDate => None,
        Code::MissingStartTime => None,
        Code::MissingEndTime => None,
        Code::TimeJumped => Some(vec![Box::new(ReplaceWithPreviousEndTime {})]),
        Code::NegativeTimeRange => None,
        Code::IncorrectDuration => Some(vec![Box::new(RecalculateDuration {})]),
    }
}

pub fn get_command_by_name(name: &str) -> Option<Box<dyn Command>> {
    match name {
        RECALCULATE_DURATION => Some(Box::new(RecalculateDuration {})),
        REPLACE_WITH_PREVIOUS_END_TIME => Some(Box::new(ReplaceWithPreviousEndTime {})),
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
