import path = require("path");
import { workspace, ExtensionContext } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  console.log("# activating...");
  const serverOptions: ServerOptions = {
    run: {
      command: "journalint",
      args: ["--stdio"],
      transport: TransportKind.stdio,
    },
    debug: {
      command: "journalint",
      args: ["--stdio"],
      transport: TransportKind.stdio,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "markdown" }],
    // synchronize: {
    //   // Notify the server about file changes to '.clientrc files contained in the workspace
    //   fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    // },
  };

  client = new LanguageClient(
    "journalint",
    "journalint",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
