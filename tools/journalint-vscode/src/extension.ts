import fs = require("fs");
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

  // Add PATH to the journalint native binary.
  let executablePath;
  if (context.extensionMode === vscode.ExtensionMode.Production) {
    // `scripts/compile.js` builds and place it into the `bundles` directory.
    // Note that `__dirname` points to the `out` directory in development and in production.
    const target = `${process.platform}-${process.arch}`;
    const projectDir = path.dirname(__dirname);
    executablePath = path.join(projectDir, "bundles", target);
  } else {
    const workspaceDir = path.dirname(path.dirname(path.dirname(__dirname)));
    executablePath = path.join(workspaceDir, "target", "debug");
  }
  outputChannel.appendLine(`Prepending [${executablePath}] to PATH.`);
  process.env.PATH = executablePath + path.delimiter + process.env.PATH;

  // Warn if there is not a native binary.
  const suffix = process.platform === "win32" ? ".exe" : "";
  const executableFullName = path.join(executablePath, "journalint" + suffix);
  if (!fs.existsSync(executableFullName)) {
    outputChannel.appendLine(
      `WARNING: Native binary not found: [${executableFullName}]`
    );
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
