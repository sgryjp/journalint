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
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::ExecuteCommandOptions;
use lsp_types::InitializeParams;
use lsp_types::NumberOrString;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::ServerCapabilities;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;

use crate::code::Code;
use crate::commands::get_command_by_name;
use crate::commands::list_available_code_actions;
use crate::commands::RECALCULATE_DURATION;
use crate::diagnostic::Diagnostic;
use crate::errors::JournalintError;
use crate::linemap::LineMap;
use crate::lint::lint;
use crate::parse::parse;

const E_UNKNOWN_COMMAND: i32 = 1;
const E_INVALID_ARGUMENTS: i32 = 2;

#[derive(Default)]
pub struct ServerState {
    pub diagnostics: HashMap<Url, Vec<Diagnostic>>,
    pub sent_requests: Vec<Request>,
    _msgid_counter: u16,
}

impl ServerState {
    fn next_request_id(&mut self) -> RequestId {
        self._msgid_counter = self._msgid_counter.wrapping_add(1);
        RequestId::from(self._msgid_counter as i32)
    }
}

pub fn service_main() -> Result<(), JournalintError> {
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
            commands: vec![RECALCULATE_DURATION.to_string()],
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
    message_loop(&conn, &init_params)?;
    io_threads.join()?;

    info!("Shutting down journalint language server.");
    Ok(())
}

fn message_loop(conn: &Connection, _init_params: &InitializeParams) -> Result<(), JournalintError> {
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
    Ok(())
}

fn on_text_document_did_open(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Notification,
) -> Result<(), JournalintError> {
    let params: DidOpenTextDocumentParams = serde_json::from_value(msg.params)?;
    let uri = params.text_document.uri;
    let content = params.text_document.text.as_str();
    let version = None;
    let diagnostics = lint_and_publish_diagnostics(conn, &uri, content, version)?;
    state.diagnostics.insert(uri, diagnostics);
    Ok(())
}

fn on_text_document_did_change(
    state: &mut ServerState,
    conn: &Connection,
    msg: lsp_server::Notification,
) -> Result<(), JournalintError> {
    let params: DidChangeTextDocumentParams = serde_json::from_value(msg.params)?;
    let uri = params.text_document.uri;
    let content = params
        .content_changes
        .last()
        .map(|e| e.text.as_str())
        .unwrap_or("");
    let version = Some(params.text_document.version);
    let diagnostics = lint_and_publish_diagnostics(conn, &uri, content, version)?;
    state.diagnostics.insert(uri, diagnostics);
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
    for d in diagnostics.iter() {
        // Simply ignore diagnostics from tools other than journalint
        // (Every code of journalint is a string which must be parsable into Code)
        let code = &d.code;
        let Some(NumberOrString::String(code)) = code else {
            continue;
        };
        let Ok(code) = str::parse::<Code>(code) else {
            continue;
        };

        // List up all available code actions for the code
        let mut commands: Vec<Command> = list_available_code_actions(&code)
            .unwrap_or_default()
            .iter()
            .map(|fix| {
                Command::new(
                    fix.title().to_string(),   // Title string presented to users
                    fix.command().to_string(), // List of commands (contribution points)
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
    let Some(command) = get_command_by_name(&params.command) else {
        let errmsg = format!("Unknown command: {}", params.command.as_str());
        conn.sender.send(Message::Response(Response::new_err(
            msg.id.clone(),
            E_UNKNOWN_COMMAND,
            errmsg.clone(),
        )))?;
        return Err(JournalintError::UnknownCommand(errmsg));
    };

    // Extract command parameters from the message
    if params.arguments.len() != 2 {
        let errmsg = format!(
            "Number of command parameters is expected to be 2 but was {}",
            params.arguments.len()
        );
        conn.sender.send(Message::Response(Response::new_err(
            msg.id.clone(),
            E_INVALID_ARGUMENTS,
            errmsg.clone(),
        )))?;
        return Err(JournalintError::UnexpectedArguments(errmsg));
    }
    let url: Url = serde_json::from_value(params.arguments[0].clone())?;
    let range: lsp_types::Range = serde_json::from_value(params.arguments[1].clone())?;

    // Execute the command
    let Some(edit) = command.execute(state, &url, &range) else {
        return Ok(()); // Do nothing if command does not change the document
    };

    // Request the changes to be executed to the client
    let request_id = state.next_request_id();
    info!("[S:{}] textDocument/applyEdit", request_id);
    let params = ApplyWorkspaceEditParams {
        label: Some(command.title().to_string()),
        edit,
    };
    let request = Request::new(request_id, "workspace/applyEdit".to_string(), params);
    conn.sender.send(Message::Request(request.clone()))?;

    // Remember the request until a corresponding response arrives
    state.sent_requests.push(request);
    Ok(())
}

fn lint_and_publish_diagnostics(
    conn: &Connection,
    uri: &Url,
    content: &str,
    version: Option<i32>,
) -> Result<Vec<Diagnostic>, JournalintError> {
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
    let diagnostics = parse_and_lint(content, Some(filename));

    // Publish them to the client
    let params = PublishDiagnosticsParams::new(
        uri.clone(),
        diagnostics
            .iter()
            .map(|d| d.clone().into())
            .collect::<Vec<lsp_types::Diagnostic>>(),
        version,
    );
    let params = serde_json::to_value(params)?;
    conn.sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        }))?;

    Ok(diagnostics)
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
