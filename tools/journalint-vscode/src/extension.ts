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

export function activate(context: vscode.ExtensionContext) {
  console.log("Activating journalint extension...");

  // Add PATH to debug build of journalint
  if (context.extensionMode !== vscode.ExtensionMode.Production) {
    const srcRoot = path.dirname(path.dirname(__dirname));
    const executablePath = path.join(srcRoot, "server", "target", "debug");
    process.env.PATH = executablePath + path.delimiter;
  }

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
    // synchronize: {
    //   // Notify the server about file changes to '.clientrc files contained in the workspace
    //   fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    // },
  };

  // Start LSP client
  client = new LanguageClient(
    "journalint",
    "journalint",
    serverOptions,
    clientOptions
  );
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  console.log(`Deactivating journalint extension...`);
  if (!client) {
    return undefined;
  }
  return client.stop();
}
