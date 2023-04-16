use core::result::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::errors::JournalintError;
use lazy_static::lazy_static;
use lsp_types::Position;
use lsp_types::Diagnostic;
use regex::Regex;

pub struct Journalint {
    diagnostics: Vec<Diagnostic>,
}

impl Journalint {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.as_ref()
    }

    /// Parse a journal file content.
    pub fn parse(&self, _doc: &str) -> Result<(), JournalintError> {
        // Roughly extract document structure

        // Parse journal entries

        // Lint
        Ok(())
    }
}

pub fn _parse_file<P: AsRef<Path>>(path: P) -> Result<(), JournalintError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
              ^-\s+
              (?P<start_time>\d\d:\d\d)-(?P<end_time>\d\d:\d\d)\s+
              ((?P<code>[A-Za-z0-9]+)\s+)?"
        )
        .unwrap();
    }
    let path = path.as_ref();
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        let line = line?; // TODO: Put line number to the error
        let Some(captures) = RE.captures(line.as_str()) else {
            return Err(JournalintError::FatalParseError {
                pos: Some(Position {
                    line: i as u32,
                    character: 0,
                }),
                path: path.into(),
                msg: format!("a"),
            });
        };
        eprintln!("{:?}", captures);
    }

    Ok(())
}
