pub mod filerepo;
pub mod model;

use std::error::Error;
use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};

pub struct Diary<'a> {
    tree: filerepo::tree::Tree,
    clock: Box<dyn Fn() -> DateTime<Utc> + 'a>,
}

impl fmt::Debug for Diary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diary").field("tree", &self.tree).finish()
    }
}

#[derive(Debug)]
pub enum DiaryError {
    FileRepoError(filerepo::tree::FileRepoError),
}

impl From<filerepo::tree::FileRepoError> for DiaryError {
    fn from(error: filerepo::tree::FileRepoError) -> DiaryError {
        DiaryError::FileRepoError(error)
    }
}

impl fmt::Display for DiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiaryError::FileRepoError(e) => write!(f, "File repository error: {}", e),
        }
    }
}

impl Error for DiaryError {}

type DiaryResult<T> = Result<T, DiaryError>;

#[derive(Debug)]
pub struct DiaryEntryKey {
    date: DateTime<Utc>,
}

impl<'a> Diary<'a> {
    pub fn open(path: &Path) -> Result<Diary<'a>, DiaryError> {
        Diary::open_custom(path, Utc::now)
    }

    pub fn open_custom<C>(path: &Path, clock: C) -> Result<Diary<'a>, DiaryError>
    where
        C: 'a,
        C: Fn() -> DateTime<Utc>,
    {
        let tree = filerepo::tree::Tree::new(&path)?;
        let diary = Diary {
            tree,
            clock: Box::new(clock),
        };
        Ok(diary)
    }

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
