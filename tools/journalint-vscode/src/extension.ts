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

import * as log from "./log";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  // Initialize an output channel for logging activity of this extension and the language server.
  const outputChannel = log.initialize();

  log.info("Activating journalint-vscode...");

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
  log.info(`Prepending [${executablePath}] to PATH.`);
  process.env.PATH = executablePath + path.delimiter + process.env.PATH;

  // Warn if the bundled native binary was not found.
  const suffix = process.platform === "win32" ? ".exe" : "";
  const executableFullName = path.join(executablePath, "journalint" + suffix);
  if (!fs.existsSync(executableFullName)) {
    log.warn(`Native binary not found: [${executableFullName}]`);
  }

  // Setup environment variables for the language server.
  if (!process.env.RUST_BACKTRACE) {
    process.env.RUST_BACKTRACE = "1";
  }
  process.env.RUST_LOG = "journalint=debug";

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
    outputChannel: outputChannel, // Share the same output channel
    documentSelector: [
      {
        scheme: "file",
        language: "markdown",
        // Target only a saved file named like "YYYY-MM-DD.md"
        pattern: "**/????-??-??.md",
      },
    ],
  };

  // Start LSP client
  client = new LanguageClient(
    "journalint",
    "journalint",
    serverOptions,
    clientOptions,
  );
  client.start();

  log.info("Activated journalint-vscode.");
}

export function deactivate(): Thenable<void> | undefined {
  try {
    log.info(`Deactivating journalint extension...`);
    if (!client) {
      return undefined;
    }
    return client.stop();
  } finally {
    log.info(`Deactivated journalint extension.`);
    log.cleanup();
  }
}
