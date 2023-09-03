import * as vscode from "vscode";

let outputChannel: vscode.OutputChannel | undefined = undefined;

export function initialize(): vscode.OutputChannel {
  if (!outputChannel) {
    // Note that this channel is shared with the journalint language server
    // so that this extension always prepends '[journalint-vscode]' to its messages.
    outputChannel = vscode.window.createOutputChannel("journalint");
  }
  return outputChannel;
}

export function cleanup() {
  if (outputChannel) {
    outputChannel.dispose();
    outputChannel = undefined;
  }
}

export function error(message: string) {
  write("ERROR", message);
}

export function warn(message: string) {
  write("WARN ", message);
}

export function info(message: string) {
  write("INFO ", message);
}

export function debug(message: string) {
  write("DEBUG", message);
}

function write(level: string, message: string) {
  if (!outputChannel) {
    return;
  }

  const now = new Date().toISOString();
  outputChannel.appendLine(`[${now} ${level} journalint-vscode] ${message}`);
}
