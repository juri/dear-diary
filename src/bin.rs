extern crate clap;

mod clidiary;
mod diarydir;
mod entryinput;

use clap::{App, Arg, SubCommand};
use clidiary::CLIDiary;
use diary_core::DiaryEntryKey;
use std::path::PathBuf;
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
    let output = make_entry_list(
        &keys,
        if enumerate {
            ListOption::Enumerate
        } else if enumerate_reverse {
            ListOption::EnumerateReverse
        } else {
            ListOption::Plain
        },
    );
    for line in output {
        println!("{}", line);
    }
}

enum ListOption {
    Enumerate,
    EnumerateReverse,
    Plain,
}

fn make_entry_list(keys: &Vec<DiaryEntryKey>, option: ListOption) -> Vec<String> {
    match option {
        ListOption::Enumerate => {
            let width = order_of_magnitude(keys.len());
            (1..)
                .zip(keys)
                .map(|(index, key): (usize, &DiaryEntryKey)| {
                    format!("{:width$} {}", index, key.to_string(), width = width)
                })
                .collect()
        }
        ListOption::EnumerateReverse => {
            let key_count = keys.len();
            let width = order_of_magnitude(key_count);
            (0..)
                .zip(keys)
                .map(|(index, key): (usize, &DiaryEntryKey)| {
                    format!(
                        "{:width$} {}",
                        key_count - index,
                        key.to_string(),
                        width = width
                    )
                })
                .collect()
        }
        ListOption::Plain => keys.iter().map(|k| format!("{}", k.to_string())).collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(&(vec![]), ListOption::Plain)
        );
    }

    #[test]
    fn enumerated_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(&(vec![]), ListOption::Enumerate)
        );
    }

    #[test]
    fn reverse_enumerated_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(&(vec![]), ListOption::EnumerateReverse)
        );
    }

    #[test]
    fn plain_entry_list_works() {
        let dts1 = "2020-07-10 09:11:00 UTC";
        let dts2 = "2020-07-10 20:51:00 UTC";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(&(vec![k1, k2]), ListOption::Plain);

        assert_eq!(vec![dts1, dts2], entry_list);
    }

    #[test]
    fn enumerated_entry_list_works() {
        let dts1 = "2020-07-10 09:11:00 UTC";
        let dts2 = "2020-07-10 20:51:00 UTC";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(&(vec![k1, k2]), ListOption::Enumerate);

        assert_eq!(
            vec![format!("1 {}", dts1), format!("2 {}", dts2)],
            entry_list
        );
    }

    #[test]
    fn reverse_enumerated_entry_list_works() {
        let dts1 = "2020-07-10 09:11:00 UTC";
        let dts2 = "2020-07-10 20:51:00 UTC";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(&(vec![k1, k2]), ListOption::EnumerateReverse);

        assert_eq!(
            vec![format!("2 {}", dts1), format!("1 {}", dts2)],
            entry_list
        );
    }
}
