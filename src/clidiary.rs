use diary_core::{Diary, DiaryEntryKey, MatchingDateBehavior, TagIndex};
use std::path::Path;
use std::process;

pub struct CLIDiary<'a> {
    pub diary: Diary<'a>,
}

impl<'a> CLIDiary<'a> {
    pub fn open(path: &Path) -> CLIDiary {
        match Diary::open(path) {
            Ok(diary) => CLIDiary { diary },
            Err(err) => {
                eprintln!("Error opening diary: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn show_entry(&self, key: &DiaryEntryKey) {
        println!("{}", self.text_for_entry(key))
    }

    pub fn text_for_entry(&self, key: &DiaryEntryKey) -> String {
        match self.diary.get_text_for_entry(key) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("Error retrieving diary entry: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn list_keys(&self) -> Vec<DiaryEntryKey> {
        match self.diary.list_keys() {
            Ok(keys) => keys,
            Err(err) => {
                eprintln!("Error listing diary content: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn add_entry(&self, entry: &str, key: Option<DiaryEntryKey>) -> DiaryEntryKey {
        let tag_index = self.open_index();
        match self
            .diary
            .add_entry(&tag_index, entry, key, MatchingDateBehavior::Append)
        {
            Ok(key) => key,
            Err(err) => {
                eprintln!("Error creating entry: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn replace_entry(&self, entry: &str, key: DiaryEntryKey) -> DiaryEntryKey {
        let tag_index = self.open_index();
        match self.diary.add_entry(
            &tag_index,
            entry,
            Some(key),
            MatchingDateBehavior::Overwrite,
        ) {
            Ok(key) => key,
            Err(err) => {
                eprintln!("Error replacing entry: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn search_tags(&self, tags: &[&str]) -> Vec<DiaryEntryKey> {
        let tag_index = self.open_index();
        match self.diary.search_tags(&tag_index, tags) {
            Ok(keys) => keys,
            Err(err) => {
                eprintln!("Error searching tags: {}", err);
                process::exit(1)
            }
        }
    }

    pub fn reindex(&self) {
        let tag_index = self.open_index();
        match self.diary.reindex(&tag_index) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error reindexing: {}", err);
                process::exit(1)
            }
        }
    }

    fn open_index(&self) -> TagIndex {
        match self.diary.open_index() {
            Ok(index) => index,
            Err(err) => {
                eprintln!("Error opening tag index: {}", err);
                process::exit(1)
            }
        }
    }
}
