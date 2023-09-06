mod arg;
mod autofix;
mod code;
mod diagnostic;
mod errors;
mod linemap;
mod lint;
mod parse;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;

use ariadne::Color;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use clap::Parser;
use env_logger::TimestampPrecision;
use log::error;
use log::info;
use lsp_server::Connection;
use lsp_server::Message;
use lsp_types::DidChangeTextDocumentParams;
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::InitializeParams;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::ServerCapabilities;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;

use crate::arg::Arguments;
use crate::code::Code;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::linemap::LineMap;

/// Entry point of journalint CLI.
fn main() -> Result<(), JournalintError> {
    let args = Arguments::parse_from(env::args());
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();
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
    let content = match read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read {}: {}", filename, e);
            return exitcode::IOERR;
        }
    };

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

fn lsp_main() -> Result<(), JournalintError> {
    info!("Starting journalint language server...");
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
    info!("Shutting down journalint language server.");
    Ok(())
}

pub fn lsp_dispatch(
    conn: &Connection,
    _init_params: &InitializeParams,
) -> Result<(), JournalintError> {
    for msg in &conn.receiver {
        match msg {
            Message::Notification(msg) => {
                if msg.method == "textDocument/didOpen" {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(msg.params)?;
                    let uri = params.text_document.uri;
                    let content = params.text_document.text.as_str();
                    let version = None;
                    run(conn, &uri, content, version)?;
                } else if msg.method == "textDocument/didChange" {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(msg.params)?;
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
            Message::Request(_) => (),
            Message::Response(_) => (),
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

    // Parse the content then convert diagnostics into the ones of corresponding LSP type
    let diagnostics = parse_and_lint(content, Some(filename))
        .iter()
        .map(|d| d.clone().into())
        .collect::<Vec<lsp_types::Diagnostic>>();

    // Publish them to the client
    let params = PublishDiagnosticsParams::new(uri.clone(), diagnostics, version);
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))
        .map_err(|e| JournalintError::LspCommunicationError(e.to_string()))?;

    Ok(())
}

fn parse_and_lint(content: &str, source: Option<&str>) -> Vec<Diagnostic> {
    let line_map = Arc::new(LineMap::new(content));

    // Parse
    let (journal, errors) = parse::parse(content);
    let mut diagnostics = errors
        .iter()
        .map(|e| {
            Diagnostic::new_warning(
                e.span(),
                Code::ParseError,
                format!("Parse error: {}", e),
                None,
                line_map.clone(),
            )
        })
        .collect::<Vec<Diagnostic>>();

    // Lint
    if let Some(journal) = journal {
        diagnostics.append(&mut lint::lint(&journal, source, line_map));
    }

    diagnostics
}

fn report(content: &str, filename: Option<&str>, diag: &Diagnostic) {
    let stdin_source_name = "<STDIN>".to_string();
    let filename = filename.unwrap_or(&stdin_source_name);
    let start = diag.span().start;
    let end = diag.span().end;
    let message = diag.message();

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
