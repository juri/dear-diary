extern crate clap;

mod diarydir;

use clap::{App, Arg};
use diary_core::{self, Diary, DiaryEntryKey};
use std::path::{Path, PathBuf};
use std::process;

pub fn main() {
    let matches = App::new("ddiary")
        .version("0.1.0")
        .author("Juri Pakaste <juri@juripakaste.fi>")
        .about("Manages diaries")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("Location of the diary directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("DIARY_NAME")
                .help("Name of the diary")
                .takes_value(true),
        )
        .get_matches();
    let mut path = matches
        .value_of("path")
        .map(PathBuf::from)
        .or_else(|| diarydir::default_dir())
        .unwrap_or_else(|| {
            eprintln!("Couldn't determine diary root directory");
            process::exit(1)
        });
    path.push(matches.value_of("name").unwrap_or("default"));
    let diary = CLIDiary::open(&path);
    diary.list_dates().first().map(|d| diary.show_entry(d));
}

struct CLIDiary {
    diary: Diary,
}

impl CLIDiary {
    fn open(path: &Path) -> CLIDiary {
        match diary_core::open(path) {
            Ok(diary) => CLIDiary { diary },
            Err(err) => {
                eprintln!("Error opening diary: {}", err);
                process::exit(1)
            }
        }
    }

    fn show_entry(&self, key: &DiaryEntryKey) {
        match self.diary.get_text_for_entry(key) {
            Ok(text) => println!("{}", text),
            Err(err) => {
                eprintln!("Error retrieving diary entry: {}", err);
                process::exit(1)
            }
        }
    }

    fn list_dates(&self) -> Vec<DiaryEntryKey> {
        match self.diary.list_dates() {
            Ok(keys) => keys,
            Err(err) => {
                eprintln!("Error listing diary content: {}", err);
                process::exit(1)
            }
        }
    }
}
