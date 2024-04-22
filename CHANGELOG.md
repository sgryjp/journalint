<!-- markdownlint-disable no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
except that "Misc" type of change is added and "Unreleased" section is removed.
This project use [Calendar Versioning](https://calver.org/) in form of
`YY.MM.build` where the `build` part is just an incrementing number. This file
is maintained using [towncrier](https://towncrier.readthedocs.io/).

<!-- towncrier release notes start -->

## [24.4.0](https://github.com/sgryjp/journalint/tree/24.4.0) - 2024-04-22

### Added

- Check start-time mismatch between front-matter and first entry.

### Changed

- Now journalint command writes diagnostic report to stdout instead of stderr.

## [24.3.0](https://github.com/sgryjp/journalint/tree/22.3.0) - 2024-03-28

### Added

- Supprt simple report format for violations by a new command line option
  `--report=oneline`.

### Misc

- Start using towncrier to maintain `CHANGELOG.md`.

## [0.2.1](https://github.com/sgryjp/journalint/tree/0.2.1) - 2024-05-02

### Added

- Support exporting journal entry data in JSON format.
  ([cccf127](https://github.com/sgryjp/journalint/commit/cccf127d465f4bfa3880914c97592364496be598))

### Fixed

- Clear diagnostics for a file on closing it.
- Fixed an issue on opening a file specified with a relative path in command
  line argument.

### Security

- Update serde_yaml to 0.9.30 (GHSA-r24f-hg58-vfrw)
- Update chrono to 0.4.31 (CVE-2020-26235)

## [0.2.0](https://github.com/sgryjp/journalint/tree/0.2.0) - 2023-12-28

### Added

- Support code action (quick fix) to automatically fix lint errors.
- Add related diagnostic information for `time-jumped` error.

## [0.1.1](https://github.com/sgryjp/journalint/tree/0.1.1) - 2023-07-14

### Added

- Initial release
