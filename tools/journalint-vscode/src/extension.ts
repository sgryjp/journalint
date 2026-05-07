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
    const targetDir = path.join(workspaceDir, "target");
    const candidates: string[] = [
      path.join(targetDir, "debug", `journalint${commandSuffix}`),
      path.join(targetDir, "release", `journalint${commandSuffix}`),
    ];

    // CI often builds into target/<triple>/{debug,release}/journalint.
    try {
      for (const entry of fs.readdirSync(targetDir, { withFileTypes: true })) {
        if (!entry.isDirectory()) {
          continue;
        }
        candidates.push(
          path.join(
            targetDir,
            entry.name,
            "debug",
            `journalint${commandSuffix}`,
          ),
        );
        candidates.push(
          path.join(
            targetDir,
            entry.name,
            "release",
            `journalint${commandSuffix}`,
          ),
        );
      }
    } catch {
      // Ignore directory scan failures and rely on default candidate paths.
    }

    const existing = candidates
      .map((p) => {
        try {
          return {
            path: p,
            mtimeMs: fs.statSync(p, { bigint: false, throwIfNoEntry: true })
              .mtimeMs,
          };
        } catch {
          return undefined;
        }
      })
      .filter((x) => x !== undefined)
      .sort((a, b) => b.mtimeMs - a.mtimeMs);

    if (existing.length === 0) {
      return undefined;
    }
    return existing[0].path;
  }
}
