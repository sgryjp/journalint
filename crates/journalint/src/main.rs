mod arg;
mod diagnostic;
mod errors;
mod journalint;
mod linemap;
mod lint;
mod parse;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use clap::Parser;
use lsp_server::Connection;
use lsp_server::Message::Notification;
use lsp_types::DidChangeTextDocumentParams;
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::InitializeParams;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::ServerCapabilities;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;

use crate::arg::Arguments;
use crate::errors::JournalintError;
use crate::journalint::parse_and_lint;

fn main() -> Result<(), JournalintError> {
    let args = Arguments::parse_from(env::args());
    if args.stdio {
        lsp_main()
    } else {
        let rc = cli_main(args);
        std::process::exit(rc);
    }
}

fn cli_main(args: Arguments) -> exitcode::ExitCode {
    let Some(filename) = args.filename else {
        return exitcode::USAGE;
    };

    let path = PathBuf::from(&filename);
    let content = match read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {}", filename, e);
            return exitcode::IOERR;
        }
    };

    parse_and_lint(content.as_str(), Some(filename)).report();
    exitcode::OK
}

fn lsp_main() -> Result<(), JournalintError> {
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

    lsp_dispatch(&conn, &init_params)?;
    io_threads.join()?;
    eprintln!("Shutting down journalint language server.");
    Ok(())
}

pub fn lsp_dispatch(
    conn: &Connection,
    _init_params: &InitializeParams,
) -> Result<(), JournalintError> {
    for msg in &conn.receiver {
        if let Notification(notif) = msg {
            if notif.method == "textDocument/didOpen" {
                let params: DidOpenTextDocumentParams = serde_json::from_value(notif.params)?;
                let uri = params.text_document.uri;
                let content = params.text_document.text.as_str();
                let version = None;
                run(conn, &uri, content, version)?;
            } else if notif.method == "textDocument/didChange" {
                let params: DidChangeTextDocumentParams = serde_json::from_value(notif.params)?;
                let uri = params.text_document.uri;
                let content = params
                    .content_changes
                    .last()
                    .map(|e| e.text.as_str())
                    .unwrap_or("");
                let version = params.text_document.version;
                run(conn, &uri, content, Some(version))?;
            }
        }
    }
    Ok(())
}

fn run(
    conn: &Connection,
    uri: &Url,
    content: &str,
    version: Option<i32>,
) -> Result<(), JournalintError> {
    // Extract filename in the given URL
    let Some(segments) = uri.path_segments() else {
        let msg = format!("failed to split into segments: {}", uri);
        return Err(JournalintError::InvalidUrl(msg));
    };
    let Some(filename) = segments.into_iter().last() else {
        let msg = format!("failed to extract last segment: {}", uri);
        return Err(JournalintError::InvalidUrl(msg));
    };
    let filename = String::from(filename);

    // Parse the content then convert diagnostics into the corresponding LSP type
    let journalint = parse_and_lint(content, Some(filename));
    let diagnostics = journalint
        .diagnostics()
        .iter()
        .map(|d| d.to_lsp_types(journalint.linemap()))
        .collect();

    // Publish them to the client
    let params = PublishDiagnosticsParams::new(uri.clone(), diagnostics, version);
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))
        .map_err(|e| JournalintError::LspCommunicationError(e.to_string()))?;

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
            let journalint = parse_and_lint(&content, Some(filename));
            let diagnostics: Vec<lsp_types::Diagnostic> = journalint
                .diagnostics()
                .iter()
                .map(|d| d.to_lsp_types(journalint.linemap()))
                .collect();
            insta::assert_yaml_snapshot!(path.file_stem().unwrap().to_str().unwrap(), diagnostics);
        }
    }
}
