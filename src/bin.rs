extern crate clap;

mod diarydir;
mod entryinput;

use clap::{App, Arg, SubCommand};
use diary_core::{Diary, DiaryEntryKey};
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
        .subcommand(SubCommand::with_name("list").about("Lists entries"))
        .subcommand(
            SubCommand::with_name("show").about("Show an entry").arg(
                Arg::with_name("date")
                    .short("d")
                    .long("date")
                    .value_name("DATE")
                    .help("Entry date")
                    .takes_value(true),
            ),
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
    let entry = entryinput::read_entry();
    match entry {
        Ok(e) if e.len() > 0 => {
            println!("Created entry with key {:?}", diary.add_entry(&e));
            ()
        }
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to read entry: {}", e);
            process::exit(1)
        }
    }
    diary.list_dates().first().map(|d| diary.show_entry(d));
}

struct CLIDiary<'a> {
    diary: Diary<'a>,
}

impl<'a> CLIDiary<'a> {
    fn open(path: &Path) -> CLIDiary {
        match Diary::open(path) {
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

    fn add_entry(&self, entry: &str) -> DiaryEntryKey {
        match self.diary.add_entry(entry) {
            Ok(key) => key,
            Err(err) => {
                eprintln!("Error creating entry: {}", err);
                process::exit(1)
            }
        }
    }
}
