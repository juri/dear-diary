use chrono::{TimeZone, Utc};
use diary_core::{Diary, DiaryEntryKey};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_tags_store() {
    let dir = tempdir().unwrap();
    let clock = || Utc.ymd(2020, 8, 30).and_hms(13, 37, 00);
    let diary = Diary::open_custom(&PathBuf::from(dir.path()), clock).unwrap();
    let index = diary.open_index().unwrap();
    let key1 = DiaryEntryKey::parse_from_string("2020-08-30 13:37 +03:00").unwrap();
    let key2 = DiaryEntryKey::parse_from_string("2020-08-31 13:37 +03:00").unwrap();
    let rkey1 = diary
        .add_entry(&index, "#(hello world)# #with #tags", Some(key1.clone()))
        .unwrap();
    let rkey2 = diary
        .add_entry(&index, "another note, #more #tags", Some(key2.clone()))
        .unwrap();
    assert_eq!(key1, rkey1);
    assert_eq!(key2, rkey2);
    let keys = diary.search_tags(&index, &vec!["tags"]).unwrap();
    assert_eq!(keys, vec![key1.clone(), key2.clone()]);
    let keys = diary.search_tags(&index, &vec!["hello world"]).unwrap();
    assert_eq!(keys, vec![key1.clone()]);
    let keys = diary.search_tags(&index, &vec!["with", "more"]).unwrap();
    assert_eq!(keys, vec![key1.clone(), key2.clone()]);
}

#[test]
fn test_tags_store_reindex() {
    let dir = tempdir().unwrap();
    let clock = || Utc.ymd(2020, 8, 30).and_hms(13, 37, 00);
    let diary = Diary::open_custom(&PathBuf::from(dir.path()), clock).unwrap();
    let index = diary.open_index().unwrap();
    let key1 = DiaryEntryKey::parse_from_string("2020-08-30 13:37 +03:00").unwrap();
    let key2 = DiaryEntryKey::parse_from_string("2020-08-31 13:37 +03:00").unwrap();
    let rkey1 = diary
        .add_entry(&index, "#(hello world)# #with #tags", Some(key1.clone()))
        .unwrap();
    let rkey2 = diary
        .add_entry(&index, "another note, #more #tags", Some(key2.clone()))
        .unwrap();
    assert_eq!(key1, rkey1);
    assert_eq!(key2, rkey2);

    diary.reindex(&index).unwrap();

    let keys = diary.search_tags(&index, &vec!["tags"]).unwrap();
    assert_eq!(keys, vec![key1.clone(), key2.clone()]);
    let keys = diary.search_tags(&index, &vec!["hello world"]).unwrap();
    assert_eq!(keys, vec![key1.clone()]);
    let keys = diary.search_tags(&index, &vec!["with", "more"]).unwrap();
    assert_eq!(keys, vec![key1.clone(), key2.clone()]);
}
