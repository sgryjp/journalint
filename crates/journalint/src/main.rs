mod arg;
mod ast;
mod code;
mod commands;
mod diagnostic;
mod errors;
mod export;
mod linemap;
mod lint;
mod parse;
mod service;
mod textedit;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use ariadne::Color;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use clap::Parser;
use commands::apply_text_edit;
use commands::Command;
use diagnostic::Diagnostic;
use env_logger::TimestampPrecision;
use errors::CliError;
use log::error;
use lsp_types::Url;

use crate::arg::Arguments;
use crate::errors::JournalintError;
use crate::lint::lint;
use crate::parse::parse;

const E_UNEXPECTED: exitcode::ExitCode = 1;

/// Entry point of journalint CLI.
fn main() -> Result<(), JournalintError> {
    // Initialize logging
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    // Parse arguments
    let args = Arguments::parse_from(env::args());

    // Start the service or the CLI
    if args.stdio {
        service::main()
    } else {
        let exit_status = match cli_main(args) {
            Ok(()) => exitcode::OK,
            Err(e) => {
                if let Some(msg) = e.message() {
                    error!("{}", msg);
                };
                e.exit_status()
            }
        };
        std::process::exit(exit_status);
    }
}

fn cli_main(args: Arguments) -> Result<(), CliError> {
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
    let (journal, mut diagnostics, line_map) = parse(&content);
    if let Some(journal) = journal.as_ref() {
        let mut d = lint(journal, &url, line_map.clone()).map_err(|e| {
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
            let (Some(ast_root), Some(command)) = (&journal, d.code().default_autofix()) else {
                continue; // unavailable
            };

            // Execute the default auto-fix command.
            let text_edit = command
                .execute(&url, ast_root, d.span())
                .map_err(|e| CliError::new(E_UNEXPECTED).with_message(e.to_string()))?;
            if let Some(text_edit) = text_edit {
                apply_text_edit(&url, text_edit)
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
                crate::export::export(fmt, journal, &mut writer).map_err(|e| {
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

#[cfg(test)]
mod snapshot_tests {
    use std::{ffi::OsStr, fs};

    use super::*;

    fn parse_and_lint(url: &Url, content: &str) -> Vec<Diagnostic> {
        let (journal, mut diagnostics, line_map) = parse(&content);
        if let Some(journal) = journal {
            let mut d = lint(&journal, &url, line_map).expect("FAILED TO LINT");
            diagnostics.append(&mut d);
        };

        diagnostics
    }

    #[test]
    fn test() {
        for entry in fs::read_dir("src/snapshots").unwrap() {
            let path = entry.and_then(|ent| ent.path().canonicalize()).unwrap();
            let path = path.as_path();
            if path.extension() != Some(OsStr::new("md")) {
                continue;
            }
            let url = &Url::from_file_path(path)
                .expect(&format!("failed to compose a URL from path: {:?}", path));
            let content = match read_to_string(path) {
                Ok(content) => content,
                Err(err) => panic!("failed to read a file: {{path: {:?}, err:{}}}", path, err),
            };

            let diagnostics = parse_and_lint(&url, &content)
                .iter()
                .map(|d| d.clone().into())
                .collect::<Vec<lsp_types::Diagnostic>>();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
