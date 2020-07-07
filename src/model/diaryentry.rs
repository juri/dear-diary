use std::collections::HashSet;

use chrono::prelude::{DateTime, Utc};

pub struct Tag(String);

pub struct DiaryEntry {
    pub date: DateTime<Utc>,
    pub heading: String,
    pub body: String,
    pub tags: HashSet<Tag>,
}
