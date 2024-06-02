use std::fs::{read_to_string, write};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use journalint_parse::ast;
use journalint_parse::diagnostic::Diagnostic;
use journalint_parse::lint::parse_and_lint;
use journalint_parse::rule::Rule;
use lsp_types::Url;

use crate::cli::arg::Arguments;
use crate::cli::export::{export, ExportFormat};
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

    // Execute specified task against the AST and diagnostics
    if args.fix {
        main_fix(&filename, &url, &content)?;
    } else if let Some(export_format) = args.export {
        main_export(
            &filename,
            &url,
            &content,
            export_format,
            args.extract_activity_prefixes,
        )?;
    } else {
        main_report(&filename, &url, &content, args.report)?;
    }

    Ok(())
}

fn main_fix(filename: &str, url: &Url, content: &str) -> Result<(), CliError> {
    let remaining_diagnostics;

    // Create a working copy of the content.
    let mut buffer = String::with_capacity(content.len() + 128);
    buffer.push_str(content);

    // Repeatedly execute parse, lint, and fix until no fix is done.
    let mut num_fixed = 0;
    'outer: loop {
        let (journal, diagnostics) = parse_and_lint(url, &buffer);
        for diagnostic in diagnostics.iter().as_ref() {
            let fixed =
                fix_violation(url, journal.as_ref(), diagnostic, &mut buffer).map_err(|e| {
                    CliError::new(E_UNEXPECTED)
                        .with_message(format!("Failed on fixing a rule violation: {e:?}"))
                })?;
            if fixed {
                num_fixed += 1;
                continue 'outer;
            }
        }
        remaining_diagnostics = diagnostics;
        break;
    }

    // Write the content back unless nothing changed
    if 0 < num_fixed {
        let path = url
            .to_file_path()
            .expect("journalint CLI does not expect to process non-local file");
        write(path, buffer).map_err(|e| {
            CliError::new(exitcode::IOERR)
                .with_message(format!("Failed on writing fixed result: {e:?}"))
        })?;
    }

    // Write remaining diagnostic report to stdout
    log::debug!("!!! {:?}", remaining_diagnostics);
    let line_map = Arc::new(LineMap::new(content)); //TODO: Stop using Arc
    for diagnostic in remaining_diagnostics {
        report(
            ReportFormat::Oneline,
            content,
            &line_map,
            Some(filename),
            &diagnostic,
            io::stdout(),
        )
        .map_err(|e| CliError::new(exitcode::IOERR).with_message(e.to_string()))?;
    }

    Ok(())
}

fn main_export(
    filename: &str,
    url: &Url,
    content: &str,
    export_format: ExportFormat,
    extract_activity_prefixes: bool,
) -> Result<(), CliError> {
    // Parse the content and lint the AST unless parsing itself failed
    let (journal, diagnostics) = parse_and_lint(url, content);

    // Write simple diagnostic report to *stderr*
    let line_map = Arc::new(LineMap::new(content)); //TODO: Stop using Arc
    for diagnostic in diagnostics {
        report(
            ReportFormat::Oneline,
            content,
            &line_map,
            Some(filename),
            &diagnostic,
            io::stderr(),
        )
        .map_err(|e| {
            CliError::new(E_UNEXPECTED)
                .with_message(format!("Failed on reporting rule violations: {e:?}"))
        })?;
    }

    // Export parsed data to stdout
    if let Some(journal) = journal {
        let mut writer = std::io::stdout();
        export(
            export_format,
            extract_activity_prefixes,
            journal,
            &mut writer,
        )
        .map_err(|e| {
            CliError::new(E_UNEXPECTED).with_message(format!("Failed to export data: {:?}", e))
        })?;
    }

    Ok(())
}

fn main_report(
    filename: &str,
    url: &Url,
    content: &str,
    report_format: ReportFormat,
) -> Result<(), CliError> {
    // Parse the content and lint the AST unless parsing itself failed
    let (_journal, diagnostics) = parse_and_lint(url, content);

    // Write diagnostic report to stdout
    let line_map = Arc::new(LineMap::new(content)); //TODO: Stop using Arc
    for diagnostic in diagnostics {
        report(
            report_format,
            content,
            &line_map,
            Some(filename),
            &diagnostic,
            io::stdout(),
        )
        .map_err(|e| CliError::new(exitcode::IOERR).with_message(e.to_string()))?;
    }

    Ok(())
}

fn fix_violation(
    url: &Url,
    journal: Option<&ast::Expr>,
    diagnostic: &Diagnostic,
    buffer: &mut String,
) -> Result<bool, JournalintError> {
    // Check if there is a default auto-fix command for the rule violation.
    let (Some(journal), Some(command)) = (journal, get_default_autofix(diagnostic.rule())) else {
        return Ok(false); // unavailable
    };

    // Execute the default auto-fix command.
    let Some(text_edit) = command.execute(url, journal, diagnostic.span())? else {
        return Ok(false);
    };

    // Apply the fix to on-memory buffer.
    text_edit.apply(buffer);
    Ok(true)
}

/// Get default auto-fix command for the rule violation.
fn get_default_autofix(rule: &Rule) -> Option<impl Command> {
    match rule {
        Rule::ParseError => None,
        Rule::MismatchedDates => Some(AutofixCommand::UseDateInFilename),
        Rule::MismatchedStartTime => None,
        Rule::MismatchedEndTime => None,
        Rule::InvalidStartTime => None,
        Rule::InvalidEndTime => None,
        Rule::MissingDate => None,
        Rule::MissingStartTime => None,
        Rule::MissingEndTime => None,
        Rule::TimeJumped => Some(AutofixCommand::ReplaceWithPreviousEndTime),
        Rule::NegativeTimeRange => None,
        Rule::IncorrectDuration => Some(AutofixCommand::RecalculateDuration),
    }
}
