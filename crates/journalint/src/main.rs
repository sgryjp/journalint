mod arg;
mod code;
mod commands;
mod diagnostic;
mod errors;
mod export;
mod linemap;
mod lint;
mod parse;
mod service;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use ariadne::Color;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use clap::Parser;
use diagnostic::Diagnostic;
use env_logger::TimestampPrecision;
use errors::CliError;
use export::ExportFormat;
use log::debug;
use log::error;
use lsp_types::Url;

use crate::arg::Arguments;
use crate::errors::JournalintError;
use crate::lint::lint;
use crate::parse::parse;

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
        CliError::new(1).with_message(format!("Failed to compose URL from path {:?}", &path))
    })?;

    // Load the content
    let content = read_to_string(&path).map_err(|e| {
        CliError::new(exitcode::IOERR).with_message(format!("Failed to read {filename:?}: {e:?}"))
    })?;

    // Parse
    let (journal, mut diagnostics, line_map) = parse(&content);
    if let Some(fmt) = args.export {
        match fmt {
            ExportFormat::Json => {
                panic!("NOT IMPLEMENTED YET")
            }
            ExportFormat::Csv => {
                panic!("NOT IMPLEMENTED YET")
            }
        }
    } else {
        // Lint the parsed content unless parsing itself failed
        if let Some(journal) = journal {
            diagnostics.append(&mut lint(&journal, &url, line_map));
        }

        if args.fix {
            // Sort diagnostics in reverse order
            diagnostics.sort_by(|a, b| b.span().start.cmp(&a.span().start));

            // Fix one by one, from the last to the first
            diagnostics.iter().map(Box::new).for_each(|d| {
                if let Err(e) = commands::fix(*d, content.as_str(), path.as_path()) {
                    debug!("Autofix failed: {e}");
                }
            });
        } else {
            // Report the diagnostics to user
            diagnostics
                .iter()
                .for_each(|d| report(&content, Some(&filename), d));
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
            diagnostics.append(&mut lint(&journal, &url, line_map));
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
