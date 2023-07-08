#![allow(dead_code)] // TODO: Remove this
mod arg;
mod diagnostic;
mod errors;
mod journalint;
mod linemap;
mod linting;
mod parsing;
mod server;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use clap::Parser;
use lsp_server::Connection;
use lsp_types::{
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use crate::arg::Arguments;
use crate::errors::JournalintError;
use crate::journalint::Journalint;
use crate::server::main_loop;

fn main() -> Result<(), JournalintError> {
    let args = Arguments::parse_from(env::args());
    if args.stdio {
        language_server_main()
    } else {
        let rc = command_main(args);
        std::process::exit(rc);
    }
}

fn command_main(args: Arguments) -> exitcode::ExitCode {
    let Some(filename) = args.filename else {
        return exitcode::USAGE;
    };

    let path = PathBuf::from(&filename);
    let content = match read_to_string(path) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Failed to read {}: {}", filename, e);
            return exitcode::IOERR;
        }
    };

    Journalint::new(Some(filename), &content).report();
    exitcode::OK
}

fn language_server_main() -> Result<(), JournalintError> {
    eprintln!("Starting journalint language server...");
    let (conn, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    })
    .unwrap();
    let init_params = conn
        .initialize(server_capabilities)
        .map_err(|e| JournalintError::LspCommunicationError(e.to_string()))?;
    let init_params: InitializeParams = serde_json::from_value(init_params)?;

    main_loop(&conn, &init_params)?;
    io_threads.join()?;
    eprintln!("Shutting down journalint language server.");
    Ok(())
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
            let journalint = Journalint::new(Some(filename), &content);
            let diagnostics = journalint.diagnostics();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
