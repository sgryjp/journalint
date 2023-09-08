use std::sync::Arc;

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

use crate::code::Code;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::linemap::LineMap;
use crate::lint::lint;
use crate::parse::parse;

pub fn service_main() -> Result<(), JournalintError> {
    info!("Starting journalint language server...");

    // Initialize connection
    let (conn, io_threads) = Connection::stdio();

    // Initialize server
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    })
    .unwrap();
    let init_params = conn.initialize(server_capabilities)?;
    let init_params: InitializeParams = serde_json::from_value(init_params)?;

    // Run the message loop
    message_loop(&conn, &init_params)?;
    io_threads.join()?;

    info!("Shutting down journalint language server.");
    Ok(())
}

fn message_loop(conn: &Connection, _init_params: &InitializeParams) -> Result<(), JournalintError> {
    // Receive messages until the connection was closed
    for msg in &conn.receiver {
        match msg {
            Message::Notification(msg) => {
                if msg.method == "textDocument/didOpen" {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(msg.params)?;
                    let uri = params.text_document.uri;
                    let content = params.text_document.text.as_str();
                    let version = None;
                    lint_and_publish_diagnostics(conn, &uri, content, version)?;
                } else if msg.method == "textDocument/didChange" {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(msg.params)?;
                    let uri = params.text_document.uri;
                    let content = params
                        .content_changes
                        .last()
                        .map(|e| e.text.as_str())
                        .unwrap_or("");
                    let version = params.text_document.version;
                    lint_and_publish_diagnostics(conn, &uri, content, Some(version))?;
                }
            }
            Message::Request(_) => (),
            Message::Response(_) => (),
        }
    }
    Ok(())
}

fn lint_and_publish_diagnostics(
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

    // Parse and lint the content
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
        }))?;

    Ok(())
}

// TODO: Let the CLI start a service and communicate with it so that it does not need to to call this function
pub fn parse_and_lint(content: &str, source: Option<&str>) -> Vec<Diagnostic> {
    let line_map = Arc::new(LineMap::new(content));

    // Parse
    let (journal, errors) = parse(content);
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
        diagnostics.append(&mut lint(&journal, source, line_map));
    }

    diagnostics
}
