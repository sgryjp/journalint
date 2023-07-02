const path = require("path");
const process = require("process");
const shell = require("shelljs");

// Build the native binary
const target = `${process.platform}_${process.arch}`;
var executablePath;
console.log("target:", target);
shell.set("-ev");
shell.exec("tsc -p ./");
shell.pushd("-q", "../..");
try {
  shell.exec("cargo build -qr");
  const stem = "journalint";
  const suffix = process.platform === "win32" ? ".exe" : "";
  executablePath = path.resolve(`target/release/${stem}${suffix}`);
} finally {
  shell.popd();
}

// Copy the native binary into "bundles' directory
shell.mkdir("-p", `bundles/${target}`);
shell.cp(executablePath, `bundles/${target}/`);
