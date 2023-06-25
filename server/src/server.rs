use lsp_server::{Connection, Message::Notification};
use lsp_types::{DidOpenTextDocumentParams, InitializeParams, PublishDiagnosticsParams};

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
                eprintln!("DidOpenTextDocumentParams: {:?}", params);
                on_text_document_did_open(conn, &params)?;
            }
        }
    }
    Ok(())
}

fn on_text_document_did_open(
    conn: &Connection,
    params: &DidOpenTextDocumentParams,
) -> Result<(), JournalintError> {
    let doc = &params.text_document;
    let Some(segments) = doc.uri.path_segments() else {
        let msg = format!("failed to split into segments: [{}]", doc.uri);
        return Err(JournalintError::Unexpected(msg.into()))
    };
    let Some(filename) = segments.into_iter().last() else {
        let msg = format!("failed to extract last segment: [{}]", doc.uri);
        return Err(JournalintError::Unexpected(msg.into()))
    };
    let filename = String::from(filename);
    let journalint = Journalint::new(Some(filename), &doc.text.as_str());

    let diagnostics = journalint
        .diagnostics()
        .into_iter()
        .map(|d| d.into_lsp_types(journalint.linemap()))
        .collect();
    let params = PublishDiagnosticsParams::new(params.text_document.uri.clone(), diagnostics, None);
    let params = serde_json::to_value(params)?;
    let result = conn.sender.send(Notification(lsp_server::Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params,
    }));
    Ok(())
}
