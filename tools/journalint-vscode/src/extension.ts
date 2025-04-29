import * as cp from "child_process";
import * as fs from "node:fs";
import * as path from "node:path";
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
  const executablePath = getExecutablePaths(
    context.extensionMode === vscode.ExtensionMode.Production,
  );
  if (!executablePath) {
    log.warn(`Native binary not found.`);
    return;
  }
  log.info(`Found native executable at ${executablePath}.`);
  process.env.PATH = executablePath + path.delimiter + process.env.PATH;

  // Setup environment variables for the language server.
  if (!process.env.RUST_BACKTRACE) {
    process.env.RUST_BACKTRACE = "1";
  }
  process.env.RUST_LOG = "journalint=debug";

  // Configure LSP client
  const serverOptions: ServerOptions = {
    run: {
      command: executablePath,
      transport: TransportKind.stdio, // --stdio will be appended by specifying this.
    },
    debug: {
      command: executablePath,
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

function getExecutablePaths(inProduction: boolean): string | undefined {
  const commandSuffix = process.platform === "win32" ? ".exe" : "";
  if (inProduction) {
    // `scripts/compile-node.{ps1,sh}` builds and place it into the `bundles` directory.
    // Note that `__dirname` points to the `out` directory in development and in production.
    const target = `${process.platform}-${process.arch}`;
    const projectDir = path.dirname(__dirname);
    return path.join(
      projectDir,
      "bundles",
      target,
      `journalint${commandSuffix}`,
    );
  } else {
    const workspaceDir = path.dirname(path.dirname(path.dirname(__dirname)));
    const executablePath = ["debug", "release"]
      // For each build configuration, compose a path to the expected executable
      .map((config) =>
        path.join(workspaceDir, "target", config, `journalint${commandSuffix}`),
      )
      // Get modified time of each executable
      .map((p) => {
        let x: [Date, string];
        try {
          x = [
            fs.statSync(p, { bigint: false, throwIfNoEntry: true }).mtime,
            p,
          ];
        } catch {
          x = [new Date(0), p];
        }
        return x;
      })
      // Keep only the one with latest modified time
      .reduce((prev, curr) => (curr[0] > prev[0] ? curr : prev));
    if (executablePath[0] === new Date(0)) {
      return undefined;
    }
    return executablePath[1];
  }
}
