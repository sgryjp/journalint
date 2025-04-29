import * as assert from "assert";
import * as path from "node:path";

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from "vscode";
import * as myExtension from "../../src/extension";

suite("Extension Test Suite", () => {
  // vscode.window.showInformationMessage("Start all tests.");

  test("Normal case", async () => {
    // Load test document
    const uri = vscode.Uri.file(
      path.resolve(__dirname, "../../../../2023-05-04.md"),
    );
    await vscode.window.showTextDocument(uri);

    // Wait for the extension to be activated and diagnostics to be ready
    while (!vscode.extensions.getExtension("sgryjp.journalint")?.isActive) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    while (vscode.languages.getDiagnostics(uri).length === 0) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    const diagnostics = vscode.languages.getDiagnostics(uri);
    const actual = new Set(
      diagnostics.map((d) => ({
        code: d.code,
        range: JSON.stringify(d.range),
      })),
    );
    const expected = new Set(
      [
        {
          code: "date-mismatch",
          range: new vscode.Range(1, 6, 1, 16),
        },
        {
          code: "endtime-mismatch",
          range: new vscode.Range(3, 5, 3, 10),
        },
        {
          code: "starttime-mismatch",
          range: new vscode.Range(2, 7, 2, 12),
        },
        {
          code: "incorrect-duration",
          range: new vscode.Range(8, 27, 8, 31),
        },
        {
          code: "negative-time-range",
          range: new vscode.Range(9, 8, 9, 13),
        },
        {
          code: "time-jumped",
          range: new vscode.Range(9, 2, 9, 7),
        },
      ].map((x) => ({ code: x.code, range: JSON.stringify(x.range) })),
    );
    assert.deepStrictEqual(expected, actual);
  });
});
