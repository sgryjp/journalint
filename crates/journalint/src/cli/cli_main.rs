use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use journalint_parse::ast;
use journalint_parse::diagnostic::Diagnostic;
use journalint_parse::lint::parse_and_lint;
use journalint_parse::violation::Violation;
use lsp_types::Url;

use crate::cli::arg::Arguments;
use crate::cli::export::export;
use crate::cli::report::{report, ReportFormat};
use crate::commands::{AutofixCommand, Command};
use crate::errors::{CliError, JournalintError};
use crate::linemap::LineMap;

const E_UNEXPECTED: exitcode::ExitCode = 1;

pub(crate) fn main(args: Arguments) -> Result<(), CliError> {
    // Make sure a filename was given
    let filename = args.filename.ok_or(
        CliError::new(exitcode::USAGE).with_message("FILENAME must be specified.".to_string()),
    )?;
    let path = PathBuf::from(&filename).canonicalize().map_err(|e| {
        CliError::new(exitcode::IOERR).with_message(format!(
            "Failed to canonicalize the filename {filename:?}: {e:?}"
        ))
    })?;
    let url = Url::from_file_path(path.clone()).map_err(|_| {
        CliError::new(E_UNEXPECTED)
            .with_message(format!("Failed to compose URL from path {:?}", &path))
    })?;

    // Load the content
    let content = read_to_string(&path).map_err(|e| {
        CliError::new(exitcode::IOERR).with_message(format!("Failed to read {filename:?}: {e:?}"))
    })?;

    // Parse the content and lint the AST unless parsing itself failed
    let (journal, mut diagnostics) = parse_and_lint(&url, &content);

    // Execute specified task against the AST and diagnostics
    if args.fix {
        // Sort diagnostics in reverse order
        diagnostics.sort_by(|a, b| b.span().start.cmp(&a.span().start));

        // Fix one by one
        for d in diagnostics.iter().as_ref() {
            fix_violation(&url, journal.as_ref(), d)
                .map_err(|e| CliError::new(E_UNEXPECTED).with_message(e.to_string()))?;
        }
    } else if let Some(export_format) = args.export {
        // Write simple diagnostic report to *stderr*
        let line_map = Arc::new(LineMap::new(&content)); //TODO: Stop using Arc
        for diagnostic in diagnostics {
            report(
                &ReportFormat::Oneline,
                &content,
                &line_map,
                Some(&filename),
                &diagnostic,
                io::stderr(),
            )
            .map_err(|e| CliError::new(E_UNEXPECTED).with_message(e.to_string()))?;
        }

        // Export parsed data to stdout
        if let Some(journal) = journal {
            let mut writer = std::io::stdout();
            export(export_format, journal, &mut writer).map_err(|e| {
                CliError::new(3).with_message(format!("Failed to export data: {:?}", e))
            })?;
        }
    } else {
        // Write diagnostic report to stdout
        let line_map = Arc::new(LineMap::new(&content)); //TODO: Stop using Arc
        for diagnostic in diagnostics {
            report(
                &args.report,
                &content,
                &line_map,
                Some(&filename),
                &diagnostic,
                io::stdout(),
            )
            .map_err(|e| CliError::new(exitcode::IOERR).with_message(e.to_string()))?;
        }
    }

    Ok(())
}

fn fix_violation(
    url: &Url,
    journal: Option<&ast::Expr>,
    diagnostic: &Diagnostic,
) -> Result<(), JournalintError> {
    // Check if there is a default auto-fix command for the diagnostic.
    let (Some(journal), Some(command)) = (journal, get_default_autofix(diagnostic.violation()))
    else {
        return Ok(()); // unavailable
    };

    // Execute the default auto-fix command.
    let text_edit = command.execute(&url, journal, diagnostic.span())?;
    if let Some(text_edit) = text_edit {
        text_edit.apply_to_file(&url)?;
    }
    Ok(())
}

/// Get default auto-fix command for the violation code.
fn get_default_autofix(violation: &Violation) -> Option<impl Command> {
    match violation {
        Violation::ParseError => None,
        Violation::MismatchedDates => Some(AutofixCommand::UseDateInFilename),
        Violation::InvalidStartTime => None,
        Violation::InvalidEndTime => None,
        Violation::MissingDate => None,
        Violation::MissingStartTime => None,
        Violation::MissingEndTime => None,
        Violation::TimeJumped => Some(AutofixCommand::ReplaceWithPreviousEndTime),
        Violation::NegativeTimeRange => None,
        Violation::IncorrectDuration => Some(AutofixCommand::RecalculateDuration),
    }
}
