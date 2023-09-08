mod arg;
mod autofix;
mod code;
mod diagnostic;
mod errors;
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
use env_logger::TimestampPrecision;
use log::error;
use service::parse_and_lint;
use service::service_main;

use crate::arg::Arguments;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;

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
        service_main()
    } else {
        let rc = cli_main(args);
        std::process::exit(rc);
    }
}

fn cli_main(args: Arguments) -> exitcode::ExitCode {
    // Make sure a filename was given
    let Some(filename) = args.filename else {
        return exitcode::USAGE;
    };

    // Load the content
    let path = PathBuf::from(&filename);
    let content = match read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read {}: {}", filename, e);
            return exitcode::IOERR;
        }
    };

    // Parse and lint it, then fix or report them
    let diagnostics = parse_and_lint(&content, Some(&filename));
    if args.fix {
        for d in diagnostics.iter() {
            if let Err(e) = autofix::fix(d, content.as_str(), path.as_path()) {
                error!("Autofix failed: {}", e)
            }
        }
    } else {
        diagnostics
            .iter()
            .for_each(|d| report(&content, Some(&filename), d));
    }

    exitcode::OK
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

    #[test]
    fn test() {
        for entry in fs::read_dir("src/snapshots").unwrap() {
            let entry = entry.unwrap();
            let path_buf = entry.path();
            let path = path_buf.as_path();
            if path.extension() != Some(OsStr::new("md")) {
                continue;
            }
            let filename = path.to_string_lossy().to_string();
            let content = read_to_string(path).unwrap();
            let diagnostics = parse_and_lint(&content, Some(&filename))
                .iter()
                .map(|d| d.clone().into())
                .collect::<Vec<lsp_types::Diagnostic>>();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
