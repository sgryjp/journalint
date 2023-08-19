use lsp_server::{Connection, Message::Notification};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
    PublishDiagnosticsParams, Url,
};

use crate::errors::JournalintError;
use crate::journalint::parse_and_lint;

pub fn main_loop(
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
        let msg = format!("failed to split into segments: [{}]", uri);
        return Err(JournalintError::Unexpected(msg));
    };
    let Some(filename) = segments.into_iter().last() else {
        let msg = format!("failed to extract last segment: [{}]", uri);
        return Err(JournalintError::Unexpected(msg));
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
