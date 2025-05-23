use std::collections::HashMap;
use std::sync::Arc;

use log::debug;
use log::error;
use log::info;
use log::warn;
use lsp_server::Connection;
use lsp_server::Message;
use lsp_server::Request;
use lsp_server::RequestId;
use lsp_server::Response;
use lsp_types::ApplyWorkspaceEditParams;
use lsp_types::CodeActionKind;
use lsp_types::CodeActionOptions;
use lsp_types::CodeActionProviderCapability;
use lsp_types::Command;
use lsp_types::DidChangeTextDocumentParams;
use lsp_types::DidCloseTextDocumentParams;
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::ExecuteCommandOptions;
use lsp_types::InitializeParams;
use lsp_types::NumberOrString;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::ServerCapabilities;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;
use lsp_types::WorkspaceEdit;
use strum::IntoEnumIterator;

use journalint_parse::ast::Expr;
use journalint_parse::diagnostic::Diagnostic;
use journalint_parse::lint::parse_and_lint;
use journalint_parse::rule::Rule;

use crate::commands::AutofixCommand;
use crate::commands::Command as _;
use crate::errors::JournalintError;
use crate::line_mapper::LineMapper;
use crate::lsptype_utils::ToLspDiagnostic;

const E_UNKNOWN_COMMAND: i32 = 1;
const E_INVALID_ARGUMENTS: i32 = 2;

/// State of the journalint language server.
#[derive(Default)]
pub struct ServerState {
    document_states: HashMap<Url, DocumentState>,
    sent_requests: Vec<Request>,
    msgid_counter: u16,
}

impl ServerState {
    /// Generates numeric ID for next request message.
    fn next_request_id(&mut self) -> RequestId {
        self.msgid_counter = self.msgid_counter.wrapping_add(1);
        RequestId::from(i32::from(self.msgid_counter))
    }

    // Get state data for the specified document.
    pub fn document_state(&self, url: &Url) -> Result<&DocumentState, JournalintError> {
        self.document_states
            .get(url)
            .ok_or_else(|| JournalintError::DocumentNotFound { url: url.clone() })
    }

    // Set state data for the specified document.
    fn set_document_state(&mut self, url: &Url, state: DocumentState) -> Option<DocumentState> {
        self.document_states.insert(url.clone(), state)
    }

    // Remove state data for the specified document.
    fn remove_document_state(&mut self, url: &Url) -> Option<DocumentState> {
        self.document_states.remove(url)
    }
}

/// State data associated with a document.
#[derive(Default)]
pub struct DocumentState {
    line_mapper: Arc<LineMapper>,
    ast_root: Option<Expr>,
}

impl DocumentState {
    pub fn new(line_mapper: Arc<LineMapper>, ast_root: Option<Expr>) -> Self {
        Self {
            line_mapper,
            ast_root,
        }
    }

    pub fn line_mapper(&self) -> Arc<LineMapper> {
        self.line_mapper.clone()
    }

    pub fn ast_root(&self) -> Option<&Expr> {
        self.ast_root.as_ref()
    }
}

pub fn main() -> Result<(), JournalintError> {
    info!("Starting journalint language server...");

    // Initialize connection
    let (conn, io_threads) = Connection::stdio();

    // Initialize server
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::new("quickfix")]),
            work_done_progress_options: lsp_types::WorkDoneProgressOptions {
                work_done_progress: Some(false),
            },
            resolve_provider: Some(false),
        })),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: AutofixCommand::iter()
                .map(|cmd| cmd.id().to_string())
                .collect::<Vec<String>>(),
            work_done_progress_options: lsp_types::WorkDoneProgressOptions {
                work_done_progress: Some(false),
            },
        }),
        ..Default::default()
    })
    .unwrap();
    let init_params = conn.initialize(server_capabilities)?;
    let init_params: InitializeParams = serde_json::from_value(init_params)?;

    // Run the message loop
    message_loop(&conn, &init_params);
    io_threads.join()?;

    info!("Shutting down journalint language server.");
    Ok(())
}

fn message_loop(conn: &Connection, _init_params: &InitializeParams) {
    let mut state = ServerState::default();

    // Receive messages until the connection was closed
    for msg in &conn.receiver {
        match msg {
            Message::Notification(msg) => {
                info!("[R] {}", msg.method);
                if msg.method == "textDocument/didOpen" {
                    // User opened a new document. Parse and lint the new document.
                    if let Err(e) = on_text_document_did_open(&mut state, conn, msg) {
                        error!("{}", e);
                    }
                } else if msg.method == "textDocument/didChange" {
                    // User modified an existing document. Parse and lint it again.
                    if let Err(e) = on_text_document_did_change(&mut state, conn, msg) {
                        error!("{}", e);
                    }
                } else if msg.method == "textDocument/didClose" {
                    // User closed a document. Clear diagnostics for the document.
                    if let Err(e) = on_text_document_did_close(&mut state, conn, msg) {
                        error!("{}", e);
                    }
                }
            }

            Message::Request(msg) => {
                info!("[R:{}] {}", msg.id, msg.method);
                if msg.method == "textDocument/codeAction" {
                    // User (client) requested a list of available code actions at a location.
                    if let Err(e) = on_text_document_code_action(&mut state, conn, msg) {
                        error!("{}", e);
                    }
                } else if msg.method == "workspace/executeCommand" {
                    // User (client) requested to execute a command.
                    if let Err(e) = on_workspace_execute_command(&mut state, conn, msg) {
                        error!("{}", e);
                    }
                } else {
                    warn!("Received an unsupported request: {}", msg.method);
                    debug!("# {:?}", msg);
                }
            }

            Message::Response(msg) => {
                // Find the request matching this response
                let index = state.sent_requests.iter().enumerate().find_map(|(i, req)| {
                    if req.id == msg.id {
                        Some(i)
                    } else {
                        None
                    }
                });

                // Forget the matching request if found
                if let Some(index) = index {
                    state.sent_requests.swap_remove(index);
                }

                // Write log message
                if let Some(result) = &msg.result {
                    info!("[R:{}] {:?}", msg.id, result);
                }
                if let Some(error) = &msg.error {
                    error!("[R:{}] {:?}", msg.id, error);
                }
            }
        }
    }
}

fn on_text_document_did_open(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Notification,
) -> Result<(), JournalintError> {
    // Deserialize parameters
    let params: DidOpenTextDocumentParams = serde_json::from_value(msg.params)?;
    let uri = params.text_document.uri;
    let content = params.text_document.text.as_str();
    let version = None;

    // Parse and lint
    let line_mapper = Arc::new(LineMapper::new(content));
    let (journal, diagnostics) = parse_and_lint(&uri, content);

    // Publish diagnostics
    publish_diagnostics(conn, &uri, &line_mapper, &diagnostics, version)?;

    // Update (replace) state data for the document
    state.set_document_state(&uri, DocumentState::new(line_mapper, journal));

    Ok(())
}

fn on_text_document_did_change(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Notification,
) -> Result<(), JournalintError> {
    // Deserialize parameters
    let params: DidChangeTextDocumentParams = serde_json::from_value(msg.params)?;
    let uri = params.text_document.uri;
    let content = params
        .content_changes
        .last()
        .map_or("", |e| e.text.as_str());
    let version = Some(params.text_document.version);

    // Parse and lint
    let line_mapper = Arc::new(LineMapper::new(content));
    let (journal, diagnostics) = parse_and_lint(&uri, content);

    // Publish diagnostics
    publish_diagnostics(conn, &uri, &line_mapper, &diagnostics, version)?;

    // Update (replace) state data for the document
    state.set_document_state(&uri, DocumentState::new(line_mapper, journal));
    Ok(())
}

fn on_text_document_did_close(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Notification,
) -> Result<(), JournalintError> {
    let params: DidCloseTextDocumentParams = serde_json::from_value(msg.params)?;
    let url = params.text_document.uri;

    // Notify client to remove the diagnostics for the document
    let params = PublishDiagnosticsParams::new(url.clone(), vec![], None);
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))?;

    // Remove state data for the document
    let _ = state.remove_document_state(&url);
    Ok(())
}

fn on_text_document_code_action(
    _state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Request,
) -> Result<(), JournalintError> {
    let params: lsp_types::CodeActionParams = serde_json::from_value(msg.params)?;
    let uri = &params.text_document.uri;
    let position = params.range;
    let diagnostics = &params.context.diagnostics;
    let mut all_commands: Vec<Command> = Vec::new();
    for d in diagnostics {
        // Determine which rule is behind the diagnosed error or warning.
        let code = &d.code;
        let Some(NumberOrString::String(code)) = code else {
            continue; // journalint's rule name is string
        };
        let Ok(rule) = str::parse::<Rule>(code) else {
            continue; // not a known journalint's rule name
        };

        // List up all available code actions for the rule violation.
        let mut commands: Vec<Command> = AutofixCommand::iter()
            .filter(|cmd| cmd.can_fix(&rule))
            .map(|cmd| {
                lsp_types::Command::new(
                    cmd.title().to_string(),
                    cmd.id().to_string(),
                    Some(vec![
                        serde_json::to_value(uri).unwrap(),
                        serde_json::to_value(position).unwrap(),
                    ]),
                )
            })
            .collect();
        all_commands.append(&mut commands);
    }
    conn.sender.send(Message::Response(Response::new_ok(
        msg.id.clone(),
        all_commands,
    )))?;
    Ok(())
}

fn on_workspace_execute_command(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Request,
) -> Result<(), JournalintError> {
    let params: lsp_types::ExecuteCommandParams = serde_json::from_value(msg.params)?;

    // Dispatch the requested command
    let Some(command) = AutofixCommand::iter().find(|cmd| cmd.id() == params.command.as_str())
    else {
        let err = format!("Unknown command: {}", params.command.as_str());
        conn.sender.send(Message::Response(Response::new_err(
            msg.id.clone(),
            E_UNKNOWN_COMMAND,
            err.clone(),
        )))?;
        return Err(JournalintError::UnknownCommand(err));
    };

    // Extract command parameters from the message
    if params.arguments.len() != 2 {
        let err = format!(
            "Number of command parameters is expected to be 2 but was {}",
            params.arguments.len()
        );
        conn.sender.send(Message::Response(Response::new_err(
            msg.id.clone(),
            E_INVALID_ARGUMENTS,
            err.clone(),
        )))?;
        return Err(JournalintError::UnexpectedArguments(err));
    }
    let url: Url = serde_json::from_value(params.arguments[0].clone())?;
    let selected_range: lsp_types::Range = serde_json::from_value(params.arguments[1].clone())?;

    // Execute the command
    let doc_state = state.document_state(&url)?;
    let line_mapper = doc_state.line_mapper();
    let ast_root = doc_state.ast_root().ok_or_else(|| {
        JournalintError::UnexpectedError(format!("No AST available for the document: {url}"))
    })?;
    let selected_span = line_mapper.lsp_range_to_span(&selected_range);
    let Some(edit) = command.execute(&url, ast_root, &selected_span)? else {
        return Ok(()); // Do nothing if command does not change the document
    };
    let text_edit = edit.to_lsp_type(&line_mapper);
    let workspace_edit = WorkspaceEdit::new(HashMap::from([(url.clone(), vec![text_edit])]));

    // Request the changes to be executed to the client
    let request_id = state.next_request_id();
    info!("[S:{}] textDocument/applyEdit", request_id);
    let params = ApplyWorkspaceEditParams {
        label: Some(command.title().to_string()),
        edit: workspace_edit,
    };
    let request = Request::new(request_id, "workspace/applyEdit".to_string(), params);
    conn.sender.send(Message::Request(request.clone()))?;

    // Remember the request until a corresponding response arrives
    state.sent_requests.push(request);
    Ok(())
}

#[warn(unused_results)]
fn publish_diagnostics(
    conn: &Connection,
    url: &Url,
    line_mapper: &Arc<LineMapper>,
    diagnostics: &[Diagnostic],
    version: Option<i32>,
) -> Result<(), JournalintError> {
    // Publish them to the client
    let params = PublishDiagnosticsParams::new(
        url.clone(),
        diagnostics
            .iter()
            .map(|d| d.clone().to_lsptype(line_mapper))
            .collect::<Vec<lsp_types::Diagnostic>>(),
        version,
    );
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))?;

    Ok(())
}
