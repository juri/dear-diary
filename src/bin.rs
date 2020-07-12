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
        .subcommand(
            SubCommand::with_name("list")
                .about("Lists entries")
                .arg(
                    Arg::with_name("enumerate")
                        .short("e")
                        .long("enumerate")
                        .help("Enumerate entries in ascending order")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("enumerate-reverse")
                        .short("E")
                        .long("enumerate-reverse")
                        .help("Enumerate entries in descending order")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("show")
                .about("Show an entry")
                .arg(
                    Arg::with_name("date")
                        .short("d")
                        .long("date")
                        .value_name("DATE")
                        .help("Entry date")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("number")
                        .short("n")
                        .long("number")
                        .value_name("NUMBER")
                        .help("Entry number (counting from first)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("reverse-number")
                        .short("N")
                        .long("reverse-number")
                        .value_name("RNUMBER")
                        .help("Entry number (counting from last)")
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
    if let Some(list_matches) = matches.subcommand_matches("list") {
        list_entries(&diary, &list_matches);
    } else if let Some(show_matches) = matches.subcommand_matches("show") {
        show_entry(&diary, &show_matches);
    } else {
        add_entry(&diary);
    }
}

fn list_entries(diary: &CLIDiary, matches: &clap::ArgMatches) {
    let keys = diary.list_keys();
    let enumerate = matches.is_present("enumerate");
    let enumerate_reverse = matches.is_present("enumerate-reverse");
    if enumerate && enumerate_reverse {
        eprintln!("Only one of enumerate and enumerate-reverse supported");
        process::exit(1)
    }
    if enumerate {
        let width = order_of_magnitude(keys.len());
        for (index, key) in (1..).zip(keys) {
            println!("{:width$} {}", index, key.to_string(), width = width);
        }
    } else if enumerate_reverse {
        let key_count = keys.len();
        let width = order_of_magnitude(key_count);
        for (index, key) in (0..).zip(keys) {
            println!(
                "{:width$} {}",
                key_count - index,
                key.to_string(),
                width = width
            );
        }
    } else {
        for key in keys.iter().map(|k| k.to_string()) {
            println!("{}", key)
        }
    }
}

fn order_of_magnitude(n: usize) -> usize {
    let nd = n as f64;
    nd.log10() as usize
}

fn show_entry(diary: &CLIDiary, matches: &clap::ArgMatches) {
    if let Some(date_param) = matches.value_of("date") {
        if let Some(key) = DiaryEntryKey::parse_from_string(date_param) {
            diary.show_entry(&key);
        } else {
            eprintln!("Failed to parse date {}", date_param);
            process::exit(1);
        }
    } else if let Some(ns) = matches.value_of("number") {
        if let Some(number) = usize::from_str_radix(ns, 10).ok() {
            let keys = diary.list_keys();
            check_entry_number(number, &keys);
            let key = &keys[number - 1];
            diary.show_entry(&key);
        } else {
            eprintln!("Failed to parse number {}", ns);
            process::exit(1);
        }
    } else if let Some(ns) = matches.value_of("reverse-number") {
        if let Some(number) = usize::from_str_radix(ns, 10).ok() {
            let keys = diary.list_keys();
            check_entry_number(number, &keys);
            let key = &keys[keys.len() - number];
            diary.show_entry(&key);
        } else {
            eprintln!("Failed to parse number {}", ns);
            process::exit(1);
        }
    }
}

fn check_entry_number(number: usize, keys: &Vec<DiaryEntryKey>) {
    if number > keys.len() {
        eprintln!("Invalid entry number {}", number);
        process::exit(1);
    }
}

fn add_entry(diary: &CLIDiary) {
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

    fn list_keys(&self) -> Vec<DiaryEntryKey> {
        match self.diary.list_keys() {
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
