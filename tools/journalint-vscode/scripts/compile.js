const path = require("path");
const process = require("process");
const shell = require("shelljs");

const targetMapping = new Map([
  ["linux-x64", "x86_64-unknown-linux-musl"],
  ["darwin-arm64", "aarch64-apple-darwin"],
  ["win32-x64", "x86_64-pc-windows-msvc"],
]);

// Resolve path to the native binary
const nodeTarget = `${process.platform}-${process.arch}`;
console.log("nodeTarget:", nodeTarget);
shell.set("-ev");
shell.exec("tsc -p ./");
const stem = "journalint";
const suffix = process.platform === "win32" ? ".exe" : "";
const rustTarget = targetMapping.get(nodeTarget);
console.log("rustTarget:", rustTarget);
const executablePath = path.resolve(
  `../../target/${rustTarget}/release/${stem}${suffix}`
);
if (!shell.test("-f", executablePath)) {
  console.log("WARNING: Native binary not found. Building...");
  shell.pushd("../../");
  shell.exec(`cargo build -qr --target ${rustTarget}`);
  shell.popd();
}

// Copy it into "bundles' directory
shell.mkdir("-p", `bundles/${nodeTarget}`);
shell.cp(executablePath, `bundles/${nodeTarget}/`);

console.log("Done.");
