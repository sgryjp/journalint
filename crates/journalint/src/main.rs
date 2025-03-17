mod cli;
mod commands;
mod errors;
mod line_mapper;
mod lsptype_utils;
mod service;
mod text_edit;

use std::env;

use clap::Parser;
use env_logger::TimestampPrecision;
use log::error;

use crate::cli::Arguments;
use crate::errors::JournalintError;

/// Entry point of journalint CLI.
fn main() -> Result<(), JournalintError> {
    // Initialize logging
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    // Parse arguments and start the service or the CLI
    let args = Arguments::parse_from(env::args());
    if args.stdio {
        service::main()
    } else {
        let exit_status = match cli::main(args) {
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

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    use std::ffi::OsStr;
    use std::fs::{self, read_to_string};
    use std::sync::Arc;

    use journalint_parse::lint::parse_and_lint;
    use lsp_types::Url;

    use crate::line_mapper::LineMapper;
    use crate::lsptype_utils::ToLspDiagnostic as _;

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

            let line_mapper = Arc::new(LineMapper::new(&content));
            let (_journal, diagnostics) = parse_and_lint(&url, &content);
            let diagnostics = diagnostics
                .iter()
                .map(|d| d.clone().to_lsptype(&line_mapper))
                .collect::<Vec<lsp_types::Diagnostic>>();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
