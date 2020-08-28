use crate::DiaryEntryKey;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TagIndex {
    root: PathBuf,
    conn: Connection,
}

pub type TagIndexResult<T> = Result<T, TagIndexError>;

#[derive(Debug)]
pub enum TagIndexError {
    DBError(rusqlite::Error),
    BadPathError(PathBuf),
    IoError(io::Error),
    IndexFormatError(String),
}

impl fmt::Display for TagIndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TagIndexError::DBError(e) => write!(f, "DB error: {}", e),
            TagIndexError::BadPathError(p) => {
                write!(f, "Bad path {}", p.to_str().unwrap_or("(no path)"))
            }
            TagIndexError::IoError(e) => write!(f, "I/O error: {}", e),
            TagIndexError::IndexFormatError(s) => write!(f, "Tag index error: {}", s),
        }
    }
}

impl From<rusqlite::Error> for TagIndexError {
    fn from(error: rusqlite::Error) -> TagIndexError {
        TagIndexError::DBError(error)
    }
}

impl From<io::Error> for TagIndexError {
    fn from(error: io::Error) -> Self {
        TagIndexError::IoError(error)
    }
}

impl From<chrono::format::ParseError> for TagIndexError {
    fn from(error: chrono::format::ParseError) -> Self {
        TagIndexError::IndexFormatError(format!("Error parsing entry key from index: {}", error))
    }
}

impl TagIndex {
    pub fn new(root: &Path) -> TagIndexResult<TagIndex> {
        let root_exists = root.exists();
        if root_exists && !root.is_dir() {
            TagIndexResult::Err(TagIndexError::BadPathError(root.to_path_buf()))
        } else {
            if !root_exists {
                fs::create_dir_all(root)?;
            }
            let index_path = root.join(PathBuf::from("index.sqlite"));
            let conn = Connection::open(&index_path)?;
            TagIndexResult::Ok(TagIndex {
                root: root.to_path_buf(),
                conn,
            })
        }
    }

    pub fn initdb(&self) -> TagIndexResult<()> {
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS tag (
                tag         TEXT NOT NULL,
                entry_key   TEXT NOT NULL,
                UNIQUE(tag, entry_key)
            )
            ",
            params![],
        )?;
        Ok(())
    }

    pub fn set_tags(&self, key: &DiaryEntryKey, tags: &[String]) -> TagIndexResult<()> {
        let db_key = entry_key_to_db_key(key);
        self.conn.execute("BEGIN TRANSACTION", params![])?;
        self.conn
            .execute("DELETE FROM tag WHERE entry_key = ?", &[&db_key])?;
        let mut stmt = self
            .conn
            .prepare("INSERT INTO tag (tag, entry_key) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(&[tag, &db_key])?;
        }
        self.conn.execute("COMMIT", params![])?;
        Ok(())
    }

    pub fn search_tags(&self, tags: &[&str]) -> TagIndexResult<Vec<DiaryEntryKey>> {
        if tags.is_empty() {
            return Ok(vec![]);
        }
        let placeholders = make_placeholders(tags.len());
        let select = format!("SELECT entry_key FROM tag WHERE tag IN ({})", placeholders);
        let mut stmt = self.conn.prepare(&select)?;
        let rows = stmt.query_map(tags, |row| row.get(0))?;
        let mut keys: Vec<DiaryEntryKey> = Vec::new();
        for key_result in rows {
            let key_str: String = key_result?;
            let key =
                DateTime::parse_from_str(&key_str, KEY_DB_FORMAT).map(|date| DiaryEntryKey {
                    date: date.with_timezone(&Utc),
                })?;
            keys.push(key);
        }
        Ok(keys)
    }
}

fn entry_key_to_db_key(key: &DiaryEntryKey) -> String {
    key.date.format(KEY_DB_FORMAT).to_string()
}

fn make_placeholders(times: usize) -> String {
    assert_ne!(times, 0);
    format!("?{}", ", ?".repeat(times - 1))
}

static KEY_DB_FORMAT: &str = "%Y%m%dT%H%M%z";
