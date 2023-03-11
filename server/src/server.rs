use lsp_server::{Connection, Message::Notification};
use lsp_types::{DidOpenTextDocumentParams, InitializeParams};

use crate::errors::JournalintError;

pub fn main_loop(conn: &Connection, init_params: &InitializeParams) -> Result<(), JournalintError> {
    for msg in &conn.receiver {
        if let Notification(notif) = msg {
            if notif.method == "textDocument/didOpen" {
                let params: DidOpenTextDocumentParams = serde_json::from_value(notif.params)?;
                on_text_document_did_open(params);
            }
        }
    }
    Ok(())
}

fn on_text_document_did_open(params: DidOpenTextDocumentParams) {
    eprintln!("DidOpenTextDocumentParams: {:?}", params);
}
