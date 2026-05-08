# Contribution Guide

## Release Process

Follow the steps below to release a new version:

1. Choose a new version number.
   - This project uses [CalVer](https://calver.org/), in form of `YY.MM.build`
     where the `build` part is just an incrementing number.
2. Update version number in the files below:
   - `Cargo.toml` and `Cargo.lock`
     - Executing `cargo build` after updating `Cargo.toml` is handy
   - `package.json` and `package-lock.json` in `tools\journalint-vscode`
     - Executing `npm version YY.MM.build` in `tools/journalint-vscode` is handy
3. Update `CHANGELOG.md`.
4. Commit the changes to `main` branch.
   - Commit log message should be `Bump version number to {NEW_VERSION_NUMBER}`
5. Create an annotated tag to the commit.
   - Tag name should be `v{NEW_VERSION_NUMBER}`.
   - Tag message is optional. (e.g.: `git tag -am ''` is okay)
6. Push the `main` branch and the tag (e.g.: `git push --follow-tags`)
