pub mod filerepo;
pub mod model;

use std::error::Error;
use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Diary {
    tree: filerepo::tree::Tree,
}

#[derive(Debug)]
pub enum DiaryError {
    FileRepoError(filerepo::tree::FileRepoError)
}

impl From<filerepo::tree::FileRepoError> for DiaryError {
    fn from(error: filerepo::tree::FileRepoError) -> DiaryError {
        DiaryError::FileRepoError(error)
    }
}

impl fmt::Display for DiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiaryError::FileRepoError(e) => write!(f, "File repository error: {}", e)
        }
    }
}

impl Error for DiaryError {}

type DiaryResult<T> = Result<T, DiaryError>;

pub struct DiaryEntryKey {
    date: DateTime<Utc>,
}

impl Diary {
    pub fn list_dates(&self) -> DiaryResult<Vec<DiaryEntryKey>> {
        self.tree
            .list() // list_dates()
            .map_err(DiaryError::from)
            .map(|dates| dates.iter().map(|d| DiaryEntryKey { date: *d }).collect())
    }

    pub fn get_text_for_entry(&self, key: &DiaryEntryKey) -> DiaryResult<String> {
        self.tree.get_text(&key.date).map_err(DiaryError::from)
    }
}

pub fn open(path: &Path) -> Result<Diary, DiaryError> {
    let tree = filerepo::tree::Tree::new(&path)?;
    let diary = Diary { tree };
    Ok(diary)
}
