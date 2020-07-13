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

impl DiaryEntryKey {
    pub fn parse_from_string(s: &str) -> Option<DiaryEntryKey> {
        DateTime::parse_from_str(s, DEFAULT_KEY_FORMAT)
            .map(|date| DiaryEntryKey {
                date: date.with_timezone(&Utc),
            })
            .ok()
    }

    pub fn to_string(&self) -> String {
        self.date.format(DEFAULT_KEY_FORMAT).to_string()
    }
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

    pub fn list_keys(&self) -> DiaryResult<Vec<DiaryEntryKey>> {
        match self.tree.list().map_err(DiaryError::from) {
            Ok(dates) => {
                let mut mdates = dates;
                mdates.sort_unstable();
                Ok(mdates.iter().map(|d| DiaryEntryKey { date: *d }).collect())
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_text_for_entry(&self, key: &DiaryEntryKey) -> DiaryResult<String> {
        self.tree.get_text(&key.date).map_err(DiaryError::from)
    }

    pub fn add_entry(
        &self,
        content: &str,
        key: Option<&DiaryEntryKey>,
    ) -> DiaryResult<DiaryEntryKey> {
        let entry_dt = key.map(|k| k.date).unwrap_or_else(|| (self.clock)());
        match self.tree.get_text(&entry_dt) {
            Ok(old_text) => {
                let full_text = format!("{}\n\n{}\n", old_text.trim_end(), content.trim_end());
                self.tree.add_entry(&entry_dt, &full_text)?;
            }
            Err(_) => {
                self.tree
                    .add_entry(&entry_dt, &(format!("{}\n", content.trim_end())))?;
            }
        }
        Ok(DiaryEntryKey { date: entry_dt })
    }
}

static DEFAULT_KEY_FORMAT: &str = "%Y-%m-%d %H:%M %z";
