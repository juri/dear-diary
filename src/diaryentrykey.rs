use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiaryEntryKey {
    pub date: DateTime<Utc>,
}

impl DiaryEntryKey {
    pub fn parse_from_string(s: &str) -> Option<DiaryEntryKey> {
        DateTime::parse_from_str(s, DEFAULT_KEY_FORMAT)
            .map(|date| DiaryEntryKey {
                date: date.with_timezone(&Utc),
            })
            .ok()
    }
}

impl fmt::Display for DiaryEntryKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.date.format(DEFAULT_KEY_FORMAT).to_string())
    }
}

static DEFAULT_KEY_FORMAT: &str = "%Y-%m-%d %H:%M %z";
