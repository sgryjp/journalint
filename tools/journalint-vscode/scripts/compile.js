var shell = require("shelljs");

shell.set("-ev");
shell.exec("tsc -p ./");
