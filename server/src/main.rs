mod arg;
mod errors;
mod parser;
mod server;

use std::path::PathBuf;

use clap::Parser;
use lsp_server::Connection;
use lsp_types::{
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use crate::errors::JournalintError;
use crate::parser::parse_file;
use crate::{arg::Args, server::main_loop};

fn main() -> Result<(), JournalintError> {
    let args = Args::parse();

    if args.stdio {
        eprintln!("Starting journalint language server...");
        let (conn, io_threads) = Connection::stdio();
        let server_capabilities = serde_json::to_value(ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..Default::default()
        })
        .unwrap();
        let init_params: InitializeParams =
            serde_json::from_value(conn.initialize(server_capabilities)?).unwrap();

        main_loop(&conn, &init_params)?;
        io_threads.join()?;

        eprintln!("Shutting down journalint language server.");
        Ok(())
    } else {
        let Some(filename) = args.filename else {
            return Err(
                
                
                JournalintError::ArgumentError(format!("FILENAME must be supplied").to_owned() 
               ));
        };
        println!("# Specified lint target is: {:?}", filename);
        let path = PathBuf::from(&filename);
        if let Err(e) = parse_file(path) {
            eprintln!("{:?}", e);
        }
        Ok(())
    }
}
