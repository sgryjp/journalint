mod cli;
mod commands;
mod errors;
mod linemap;
mod lsptype_utils;
mod service;
mod textedit;

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

    use journalint_parse::diagnostic::Diagnostic;
    use journalint_parse::lint::lint;
    use journalint_parse::parse::parse;
    use lsp_types::Url;

    use crate::linemap::LineMap;
    use crate::lsptype_utils::ToLspDisgnostic;

    fn parse_and_lint(url: &Url, content: &str) -> Vec<Diagnostic> {
        let (journal, parse_errors) = parse(&content);
        let mut diagnostics: Vec<Diagnostic> = parse_errors.iter().map(Diagnostic::from).collect();
        if let Some(journal) = journal {
            let mut d = lint(&journal, &url).expect("FAILED TO LINT");
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

            let line_map = Arc::new(LineMap::new(&content));
            let diagnostics = parse_and_lint(&url, &content)
                .iter()
                .map(|d| d.clone().to_lsptype(&line_map))
                .collect::<Vec<lsp_types::Diagnostic>>();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
