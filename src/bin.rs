extern crate clap;

use clap::{Arg, App, SubCommand};

use diary_core::{self, Diary, DiaryEntryKey};

use std::process;

pub fn main() {
    let matches = App::new("ddiary")
        .version("0.1.0")
        .author("Juri Pakaste <juri@juripakaste.fi>")
        .about("Manages diaries")
        .arg(Arg::with_name("path")
            .short("p")
            .long("path")
            .value_name("PATH")
            .help("Location of the diary directory")
            .takes_value(true))
        .get_matches();

    let diary = CLIDiary::open();
    diary.list_dates().first().map(|d| diary.show_entry(d));
}

struct CLIDiary {
    diary: Diary
}

impl CLIDiary {
    fn open() -> CLIDiary {
        match diary_core::open("/tmp/diarytest") {
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


