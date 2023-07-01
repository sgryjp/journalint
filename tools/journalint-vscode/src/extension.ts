import path = require("path");
import process = require("process");
import * as vscode from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
  // Create output channel for logging activity of this extension
  outputChannel = vscode.window.createOutputChannel("journalint-vscode");

  outputChannel.appendLine("Activating journalint-vscode...");

  // Add PATH to the bundled journalint native binary.
  // (`scripts/compile.js` builds and place it into the `bundles` directory.)
  // Note that `__dirname` points to the `out` directory in development and in production.
  const executablePath = path.join(path.dirname(__dirname), "bundles", "bin");
  outputChannel.appendLine(`Appending [${executablePath}] to PATH.`);
  process.env.PATH = executablePath + path.delimiter;

  // Configure LSP client
  const serverOptions: ServerOptions = {
    run: {
      command: "journalint",
      transport: TransportKind.stdio, // --stdio will be appended by specifying this.
    },
    debug: {
      command: "journalint",
      transport: TransportKind.stdio, // --stdio will be appended by specifying this.
    },
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "markdown" }],
  };

  // Start LSP client
  client = new LanguageClient(
    "journalint",
    "journalint",
    serverOptions,
    clientOptions
  );
  client.start();
  outputChannel.appendLine("Activated journalint-vscode.");
}

export function deactivate(): Thenable<void> | undefined {
  try {
    outputChannel.appendLine(`Deactivating journalint extension...`);
    if (!client) {
      return undefined;
    }
    return client.stop();
  } finally {
    outputChannel.appendLine(`Deactivated journalint extension.`);
    outputChannel.dispose();
  }
}
