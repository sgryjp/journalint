use core::result::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::errors::JournalintError;
use chrono::NaiveTime;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct JournalEntry {
    pub position: Position,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub activity: String,
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<(), JournalintError> {
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
            return Err(JournalintError::ParseError { pos: Position{line: i, column: 0}, path: path.into(), msg: format!("a") });
        };
        eprintln!("{:?}", captures);
    }

    Ok(())
}
