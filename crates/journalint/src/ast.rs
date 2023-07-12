use core::ops::Range;
use core::result::Result;

use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

use crate::errors::JournalintError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Journal {
    front_matter: FrontMatter,
    entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn new(front_matter: FrontMatter, entries: Vec<JournalEntry>) -> Self {
        Self {
            front_matter,
            entries,
        }
    }

    pub fn front_matter(&self) -> &FrontMatter {
        &self.front_matter
    }

    pub fn entries(&self) -> &[JournalEntry] {
        self.entries.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontMatter {
    date: LooseDate,
    start: Option<LooseTime>,
    end: Option<LooseTime>,
}

impl FrontMatter {
    pub fn new(
        date: LooseDate,
        start_time: Option<LooseTime>,
        end_time: Option<LooseTime>,
    ) -> Self {
        Self {
            date,
            start: start_time,
            end: end_time,
        }
    }

    pub fn date(&self) -> &LooseDate {
        &self.date
    }

    pub fn start(&self) -> Option<&LooseTime> {
        self.start.as_ref()
    }

    pub fn end(&self) -> Option<&LooseTime> {
        self.end.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontMatterItem {
    Date(LooseDate),
    StartTime(LooseTime),
    EndTime(LooseTime),
}

#[derive(Debug, PartialEq)]
pub enum Line {
    Entry(JournalEntry),
    Misc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    time_range: LooseTimeRange,
    codes: Vec<Code>,
    duration: Duration,
    description: Description,
    span: Range<usize>,
}

impl JournalEntry {
    pub fn new(
        time_range: LooseTimeRange,
        codes: Vec<Code>,
        duration: Duration,
        description: Description,
        span: Range<usize>,
    ) -> Self {
        Self {
            time_range,
            codes,
            duration,
            description,
            span,
        }
    }

    pub fn time_range(&self) -> &LooseTimeRange {
        &self.time_range
    }

    pub fn codes(&self) -> &[Code] {
        self.codes.as_ref()
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn description(&self) -> &Description {
        &self.description
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseDate {
    value: NaiveDate,
    span: Range<usize>,
}

impl LooseDate {
    pub fn new(value: NaiveDate, span: Range<usize>) -> Self {
        LooseDate { value, span }
    }

    pub fn value(&self) -> NaiveDate {
        self.value
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTimeRange {
    start: LooseTime,
    end: LooseTime,
    span: Range<usize>,
}

impl LooseTimeRange {
    pub fn new(start: LooseTime, end: LooseTime, span: Range<usize>) -> Self {
        Self { start, end, span }
    }

    pub fn end(&self) -> &LooseTime {
        &self.end
    }

    pub fn start(&self) -> &LooseTime {
        &self.start
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseTime {
    hour: u32,
    minute: u32,
    span: Range<usize>,
}

impl LooseTime {
    pub fn new(hour: u32, minute: u32, span: Range<usize>) -> Self {
        LooseTime { hour, minute, span }
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn minute(&self) -> u32 {
        self.minute
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn to_datetime(&self, date: &LooseDate) -> Option<NaiveDateTime> {
        let day = self.hour / 24;
        let hour = self.hour - day * 24;
        let min = self.minute;
        NaiveDateTime::new(date.value, NaiveTime::from_hms_opt(hour, min, 0).unwrap())
            .checked_add_days(Days::new(day as u64))
    }
}

impl TryFrom<LooseTime> for chrono::NaiveTime {
    type Error = JournalintError;

    fn try_from(value: LooseTime) -> Result<Self, Self::Error> {
        let (minute, h) = (value.minute % 60, value.minute - ((value.minute % 60) * 60));
        let hour = value.hour + h;
        match chrono::NaiveTime::from_hms_opt(hour, minute, 0) {
            Some(t) => Ok(t),
            None => Err(JournalintError::OutOfRangeTime { hour, minute }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duration {
    value: std::time::Duration,
    span: Range<usize>,
}

impl Duration {
    pub const fn from_secs(secs: u64, span: Range<usize>) -> Duration {
        let value = std::time::Duration::from_secs(secs);
        Duration { value, span }
    }

    pub const fn value(&self) -> &std::time::Duration {
        &self.value
    }

    pub const fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    value: String,
    span: Range<usize>,
}

impl Code {
    pub fn new(value: String, span: Range<usize>) -> Self {
        Self { value, span }
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    // TODO: support categories
    activity: String,
    span: Range<usize>,
}

impl Description {
    pub fn new(activity: String, span: Range<usize>) -> Self {
        Self { activity, span }
    }

    pub fn activity(&self) -> &str {
        self.activity.as_ref()
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}
