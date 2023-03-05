use lsp_server::Connection;
use lsp_types::InitializeParams;

use crate::errors::JournalintError;

pub fn main_loop(conn: &Connection, init_params: &InitializeParams) -> Result<(), JournalintError> {
    for msg in &conn.receiver {
        println!("# msg: {:?}", msg);
    }
    Ok(())
}
