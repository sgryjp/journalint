use std::fs::read_to_string;
use std::path::PathBuf;

use ariadne::{Color, Label, Report, ReportKind, Source};
use journalint_parse::diagnostic::Diagnostic;
use journalint_parse::lint::lint;
use journalint_parse::parse::parse;
use journalint_parse::violation::Violation;
use lsp_types::Url;

use crate::cli::arg::Arguments;
use crate::cli::export::export;
use crate::commands::{AutofixCommand, Command};
use crate::errors::CliError;

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
    let (journal, parse_errors) = parse(&content);
    let mut diagnostics: Vec<Diagnostic> = parse_errors.iter().map(Diagnostic::from).collect();
    if let Some(journal) = journal.as_ref() {
        let mut d = lint(journal, &url).map_err(|e| {
            CliError::new(E_UNEXPECTED).with_message(format!("Failed on linting: {e:?}"))
        })?;
        diagnostics.append(&mut d);
    }

    // Execute specified task against the AST and diagnostics
    if args.fix {
        // Sort diagnostics in reverse order
        diagnostics.sort_by(|a, b| b.span().start.cmp(&a.span().start));

        // Fix one by one
        for d in diagnostics.iter().as_ref() {
            // Check if there is a default auto-fix command for the diagnostic.
            let (Some(ast_root), Some(command)) = (&journal, get_default_autofix(d.violation()))
            else {
                continue; // unavailable
            };

            // Execute the default auto-fix command.
            let text_edit = command
                .execute(&url, ast_root, d.span())
                .map_err(|e| CliError::new(E_UNEXPECTED).with_message(e.to_string()))?;
            if let Some(text_edit) = text_edit {
                text_edit
                    .apply_to_file(&url)
                    .map_err(|e| CliError::new(E_UNEXPECTED).with_message(e.to_string()))?;
            }
        }
    } else {
        // Write diagnostic report to stderr
        diagnostics
            .iter()
            .for_each(|d| report(&content, Some(&filename), d));

        // Export parsed data to stdout
        if let Some(fmt) = args.export {
            if let Some(journal) = journal {
                let mut writer = std::io::stdout();
                export(fmt, journal, &mut writer).map_err(|e| {
                    CliError::new(3).with_message(format!("Failed to export data: {:?}", e))
                })?;
            }
        }
    }

    Ok(())
}

/// Write a human readable report of a diagnostic
fn report(content: &str, filename: Option<&str>, diagnostic: &Diagnostic) {
    let stdin_source_name = "<STDIN>".to_string();
    let filename = filename.unwrap_or(&stdin_source_name);
    let start = diagnostic.span().start;
    let end = diagnostic.span().end;
    let message = diagnostic.message();

    Report::build(ReportKind::Error, filename, start)
        .with_message(message)
        .with_label(
            Label::new((filename, start..end))
                .with_color(Color::Red)
                .with_message(message),
        )
        .finish()
        .eprint((filename, Source::from(content)))
        .unwrap();
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
