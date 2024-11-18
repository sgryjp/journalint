# Contribution Guide

## Pull Request and News Fragment

Every pull request should contain a "news fragment" file which is essentially an
entry of `CHANGELOG.md`.

- A news fragment file:
  - Must be placed under `changelog.d` directory.
  - Must be named in format `{id}.{type}.txt`.
    - `{id}` must be an issue ID which is resolved by the PR. If there is no
      issue, use ID of the pull request itself.
    - `{type}` must be one of `security`, `removed`, `deprecated`, `added`,
      `changed`, `fixed` and `misc`
    - If no appropriate issue nor pull request exist, add prefix `+` to the file
      name. (e.g.: `+fix-usage-message-typo.fix.txt`)
- The content of a news fragment file:
  - Must start with a concise sentence describing the change.
  - May be written in markdown format.
  - May have detailed multi-line description which is separated by an empty line
    from the first line.

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
   - Run `towncrier build --yes --version {NEW_VERSION_NUMBER}`
   - ([pipx](https://github.com/pypa/pipx) is handy to install towncrier)
4. Commit the changes to `main` branch.
   - Commit log message should be `Bump version number to {NEW_VERSION_NUMBER}`
5. Create an annotated tag to the commit.
   - Tag name should be `v{NEW_VERSION_NUMBER}`.
   - Tag message is optional. (e.g.: `git tag -am ''` is okay)
6. Push the `main` branch and the tag (e.g.: `git push --follow-tags`)

## Maintaining CHANGELOG.md

This project uses [towncrier](https://towncrier.readthedocs.io/) to maintain
[`CHANGELOG.md`](CHANGELOG.md) since v22.3.0. It can read contents of "news
fragments" stored in [`changelog.d`](changelog.d), insert them into the
`CHANGELOG.md` and remove the new fragment files.
