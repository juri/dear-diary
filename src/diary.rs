use crate::diaryentrykey::DiaryEntryKey;
use crate::filerepo;
use crate::index::tags::{TagIndex, TagIndexError};
use crate::tagparser;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::path::Path;

pub struct Diary<'a> {
    clock: Box<dyn Fn() -> DateTime<Utc> + 'a>,
    tree: filerepo::tree::Tree,
}

impl fmt::Debug for Diary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diary").field("tree", &self.tree).finish()
    }
}

#[derive(Debug)]
pub enum DiaryError {
    FileRepoError(filerepo::tree::FileRepoError),
    TagIndexError(TagIndexError),
}

impl From<filerepo::tree::FileRepoError> for DiaryError {
    fn from(error: filerepo::tree::FileRepoError) -> DiaryError {
        DiaryError::FileRepoError(error)
    }
}

impl From<TagIndexError> for DiaryError {
    fn from(error: TagIndexError) -> DiaryError {
        DiaryError::TagIndexError(error)
    }
}

impl fmt::Display for DiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiaryError::FileRepoError(e) => write!(f, "File repository error: {}", e),
            DiaryError::TagIndexError(e) => write!(f, "Tag index error: {}", e),
        }
    }
}

impl Error for DiaryError {}

type DiaryResult<T> = Result<T, DiaryError>;

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
            clock: Box::new(clock),
            tree,
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
        tag_index: &TagIndex,
        content: &str,
        key: Option<DiaryEntryKey>,
        matching_date_behavior: MatchingDateBehavior,
    ) -> DiaryResult<DiaryEntryKey> {
        let key = key.unwrap_or_else(|| DiaryEntryKey {
            date: (self.clock)(),
        });
        let entry_dt = key.date;
        self.save_tags(tag_index, &key, content)?;
        let formatted_content = format!("{}\n", content.trim_end());
        match matching_date_behavior {
            MatchingDateBehavior::Overwrite => {
                self.tree.add_entry(&entry_dt, &formatted_content)?
            }
            MatchingDateBehavior::Append => match self.tree.get_text(&entry_dt) {
                Ok(old_text) => {
                    let full_text = format!("{}\n\n{}", old_text.trim_end(), &formatted_content);
                    self.tree.add_entry(&entry_dt, &full_text)?;
                }
                Err(_) => {
                    self.tree.add_entry(&entry_dt, &formatted_content)?;
                }
            },
        }
        Ok(DiaryEntryKey { date: entry_dt })
    }

    pub fn search_tags(
        &self,
        tag_index: &TagIndex,
        tags: &[&str],
    ) -> DiaryResult<Vec<DiaryEntryKey>> {
        let keys = tag_index.search_tags(tags)?;
        Ok(keys)
    }

    pub fn open_index(&self) -> DiaryResult<TagIndex> {
        let tag_index = TagIndex::new(&self.tree.root)?;
        tag_index.initdb()?;
        Ok(tag_index)
    }

    pub fn reindex(&self, tag_index: &TagIndex) -> DiaryResult<()> {
        let entry_dates = self.tree.list()?;
        let keys = entry_dates.iter().map(|d| DiaryEntryKey { date: *d }); //.collect();
        let key_tag_results = keys.map(|key| -> DiaryResult<(DiaryEntryKey, Vec<String>)> {
            let text = self.get_text_for_entry(&key)?;
            let tags = tagparser::find_tags(&text);
            Ok((key, tags))
        });
        let keys_tags = key_tag_results
            .into_iter()
            .collect::<DiaryResult<Vec<(DiaryEntryKey, Vec<String>)>>>()?;
        tag_index.recreate_index(&keys_tags)?;
        Ok(())
    }

    fn save_tags(&self, tag_index: &TagIndex, key: &DiaryEntryKey, text: &str) -> DiaryResult<()> {
        let tags = tagparser::find_tags(text);
        tag_index.set_tags(key, &tags)?;
        Ok(())
    }
}

pub enum MatchingDateBehavior {
    Overwrite,
    Append,
}
