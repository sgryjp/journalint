use lsp_server::{Connection, Message::Notification};
use lsp_types::{DidOpenTextDocumentParams, InitializeParams};

use crate::errors::JournalintError;

pub fn main_loop(
    conn: &Connection,
    _init_params: &InitializeParams,
) -> Result<(), JournalintError> {
    for msg in &conn.receiver {
        if let Notification(notif) = msg {
            if notif.method == "textDocument/didOpen" {
                _ = on_text_document_did_open(notif.params); // TODO:
            }
        }
    }
    Ok(())
}

fn on_text_document_did_open(params: serde_json::Value) -> Result<(), JournalintError> {
    let params: DidOpenTextDocumentParams = serde_json::from_value(params)?;
    eprintln!("DidOpenTextDocumentParams: {:?}", params);
    Ok(())
}
