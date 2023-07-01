var shell = require("shelljs");

shell.set("-ev");
shell.exec("tsc -p ./");
shell.exec("cargo install -qf --path ../../crates/journalint --no-track --root bundles");
