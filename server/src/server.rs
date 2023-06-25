use lsp_server::{Connection, Message::Notification};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
    PublishDiagnosticsParams, Url,
};

use crate::errors::JournalintError;
use crate::journalint::Journalint;

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
                lint(conn, &uri, content)?;
            } else if notif.method == "textDocument/didChange" {
                let params: DidChangeTextDocumentParams = serde_json::from_value(notif.params)?;
                eprintln!("DidChangeTextDocumentParams: {:?}", params);
                let uri = params.text_document.uri;
                let Some(content) = params.content_changes.last().map(|e| e.text.as_str()) else {
                    return Err(JournalintError::Unexpected("No content in textDocument/didChange notification.".into()));
                };
                lint(conn, &uri, content)?;
            }
        }
    }
    Ok(())
}

fn lint(conn: &Connection, uri: &Url, content: &str) -> Result<(), JournalintError> {
    // Extract filename in the given URL
    let Some(segments) = uri.path_segments() else {
        let msg = format!("failed to split into segments: [{}]", uri);
        return Err(JournalintError::Unexpected(msg.into()))
    };
    let Some(filename) = segments.into_iter().last() else {
        let msg = format!("failed to extract last segment: [{}]", uri);
        return Err(JournalintError::Unexpected(msg.into()))
    };
    let filename = String::from(filename);

    // Parse the content then convert diagnostics into the corresponding LSP type
    let journalint = Journalint::new(Some(filename), content);
    let diagnostics = journalint
        .diagnostics()
        .into_iter()
        .map(|d| d.into_lsp_types(journalint.linemap()))
        .collect();

    // Publish them to the client
    let params = PublishDiagnosticsParams::new(uri.clone(), diagnostics, None);
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))
        .map_err(|e| JournalintError::LspCommunicationError(e.to_string()))?;

    Ok(())
}
