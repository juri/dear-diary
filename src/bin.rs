extern crate clap;

mod clidiary;
mod diarydir;
mod entryinput;

use chrono::prelude::*;
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
            Arg::with_name(args::opts::PATH)
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("Location of the diary directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(args::opts::NAME)
                .short("n")
                .long("name")
                .value_name("DIARY_NAME")
                .help("Name of the diary")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name(args::add::SUBCOMMAND)
                .about("Add a diary entry")
                .arg(
                    Arg::with_name(args::add::STDIN)
                        .short("s")
                        .long("stdin")
                        .help("Read entry from stdin")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(args::add::DATE)
                        .short("d")
                        .long("date")
                        .value_name("DATE")
                        .help("Date for the new entry (defaults to creation time)")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name(args::edit::SUBCOMMAND)
                .about("Edit or replace a diary entry")
                .arg(
                    Arg::with_name(args::edit::STDIN)
                        .short("s")
                        .long("stdin")
                        .help("Read entry from stdin")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(args::edit::DATE)
                        .short("d")
                        .long("date")
                        .value_name("DATE")
                        .help("Date for the entry to edit or replace")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name(args::list::SUBCOMMAND)
                .about("Lists entries")
                .arg(
                    Arg::with_name(args::list::ENUM)
                        .short("e")
                        .long("enumerate")
                        .help("Enumerate entries in ascending order")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(args::list::ENUM_REVERSE)
                        .short("E")
                        .long("enumerate-reverse")
                        .help("Enumerate entries in descending order")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(args::list::SORT_REVERSE)
                        .short("S")
                        .long("sort-reverse")
                        .help("Sort latest entry first")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name(args::show::SUBCOMMAND)
                .about("Show an entry. Without extra options will display the latest one.")
                .arg(
                    Arg::with_name(args::show::DATE)
                        .short("d")
                        .long("date")
                        .value_name("DATE")
                        .help("Entry date")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name(args::show::NUMBER)
                        .short("n")
                        .long("number")
                        .value_name("NUMBER")
                        .help("Entry number (counting from first)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name(args::show::NUMBER_REVERSE)
                        .short("N")
                        .long("reverse-number")
                        .value_name("RNUMBER")
                        .help("Entry number (counting from last)")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name(args::tags::SUBCOMMAND)
                .about("Operate on tags")
                .arg(
                    Arg::with_name(args::tags::SEARCH)
                        .short("s")
                        .long("search")
                        .value_name("TAGS")
                        .help("Tags to search for")
                        .multiple(true),
                )
                .arg(
                    Arg::with_name(args::tags::REINDEX)
                        .short("I")
                        .long("index")
                        .help("Recreate tag index"),
                ),
        )
        .get_matches();
    let mut path = matches
        .value_of(args::opts::PATH)
        .map(PathBuf::from)
        .or_else(diarydir::default_dir)
        .unwrap_or_else(|| {
            eprintln!("Couldn't determine diary root directory");
            process::exit(1)
        });
    path.push(matches.value_of(args::opts::NAME).unwrap_or("default"));
    let diary = CLIDiary::open(&path);
    if let Some(list_matches) = matches.subcommand_matches(args::list::SUBCOMMAND) {
        list_entries(&diary, &list_matches);
    } else if let Some(show_matches) = matches.subcommand_matches(args::show::SUBCOMMAND) {
        show_entry(&diary, &show_matches);
    } else if let Some(add_matches) = matches.subcommand_matches(args::add::SUBCOMMAND) {
        add_entry_with_args(&diary, &add_matches);
    } else if let Some(edit_matches) = matches.subcommand_matches(args::edit::SUBCOMMAND) {
        edit_entry_with_args(&diary, &edit_matches);
    } else if let Some(tags_matches) = matches.subcommand_matches(args::tags::SUBCOMMAND) {
        tags_with_args(&diary, &tags_matches)
    }
}

fn list_entries(diary: &CLIDiary, matches: &clap::ArgMatches) {
    let keys = diary.list_keys();
    let enumerate = matches.is_present(args::list::ENUM);
    let enumerate_reverse = matches.is_present(args::list::ENUM_REVERSE);
    if enumerate && enumerate_reverse {
        eprintln!("Only one of enumerate and enumerate-reverse supported");
        process::exit(1)
    }
    let ordering = if matches.is_present(args::list::SORT_REVERSE) {
        KeyOrdering::LatestFirst
    } else {
        KeyOrdering::EarliestFirst
    };
    let output = make_entry_list(
        &keys,
        if enumerate {
            ListOption::Enumerate
        } else if enumerate_reverse {
            ListOption::EnumerateReverse
        } else {
            ListOption::Plain
        },
        ordering,
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

fn make_entry_list(
    keys: &[DiaryEntryKey],
    option: ListOption,
    ordering: KeyOrdering,
) -> Vec<String> {
    let mut entries = match option {
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
        ListOption::Plain => keys.iter().map(|k| k.to_string()).collect(),
    };
    match ordering {
        KeyOrdering::EarliestFirst => entries,
        KeyOrdering::LatestFirst => {
            entries.reverse();
            entries
        }
    }
}

enum KeyOrdering {
    LatestFirst,
    EarliestFirst,
}

fn order_of_magnitude(n: usize) -> usize {
    let nd = n as f64;
    nd.log10() as usize
}

fn show_entry(diary: &CLIDiary, matches: &clap::ArgMatches) {
    if let Some(date_param) = matches.value_of(args::show::DATE) {
        diary.show_entry(&parse_date_param(date_param));
    } else if let Some(ns) = matches.value_of(args::show::NUMBER) {
        if let Ok(number) = usize::from_str_radix(ns, 10) {
            let keys = diary.list_keys();
            check_entry_number(number, &keys);
            let key = &keys[number - 1];
            diary.show_entry(&key);
        } else {
            eprintln!("Failed to parse number {}", ns);
            process::exit(1);
        }
    } else if let Some(ns) = matches.value_of(args::show::NUMBER_REVERSE) {
        if let Ok(number) = usize::from_str_radix(ns, 10) {
            let keys = diary.list_keys();
            check_entry_number(number, &keys);
            let key = &keys[keys.len() - number];
            diary.show_entry(&key);
        } else {
            eprintln!("Failed to parse number {}", ns);
            process::exit(1);
        }
    } else {
        let keys = diary.list_keys();
        if let Some(key) = keys.last() {
            diary.show_entry(&key);
        }
    }
}

fn parse_date_param(s: &str) -> DiaryEntryKey {
    if let Some(key) = DiaryEntryKey::parse_from_string(s) {
        key
    } else if let Some(key) = parse_local_date(s) {
        key
    } else {
        eprintln!("Failed to parse date {}", s);
        process::exit(1);
    }
}

fn parse_local_date(s: &str) -> Option<DiaryEntryKey> {
    parse_local_datetime(s).map(|ldt| DiaryEntryKey {
        date: ldt.with_timezone(&Utc),
    })
}

fn parse_local_datetime(s: &str) -> Option<DateTime<Local>> {
    Local
        .datetime_from_str(s, "%Y-%m-%d %H:%M")
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|nd| nd.and_hms(12, 0, 0))
                .and_then(|ndt| Local.from_local_datetime(&ndt).latest())
        })
}

fn check_entry_number(number: usize, keys: &[DiaryEntryKey]) {
    if number > keys.len() {
        eprintln!("Invalid entry number {}", number);
        process::exit(1);
    }
}

fn add_entry_with_args(diary: &CLIDiary, matches: &clap::ArgMatches) {
    let editor = if matches.is_present(args::add::STDIN) {
        AddEditor::Stdin
    } else {
        AddEditor::Environment
    };
    let key = matches.value_of(args::add::DATE).map(parse_date_param);
    add_entry(diary, editor, key)
}

fn add_entry(diary: &CLIDiary, editor: AddEditor, key: Option<DiaryEntryKey>) {
    let entry = match editor {
        AddEditor::Stdin => entryinput::read_from_stdin(),
        AddEditor::Environment => entryinput::read_from_editor(&""),
    };
    match entry {
        Ok(e) if !e.trim().is_empty() => {
            println!("Created entry with key {:?}", diary.add_entry(&e, key));
        }
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to read entry: {}", e);
            process::exit(1)
        }
    }
}

fn edit_entry_with_args(diary: &CLIDiary, matches: &clap::ArgMatches) {
    let editor = if matches.is_present(args::edit::STDIN) {
        AddEditor::Stdin
    } else {
        AddEditor::Environment
    };
    let key = match matches.value_of(args::add::DATE).map(parse_date_param) {
        Some(k) => k,
        None => {
            eprintln!("Required date parameter not found");
            process::exit(1)
        }
    };
    edit_entry(diary, editor, key)
}

fn edit_entry(diary: &CLIDiary, editor: AddEditor, key: DiaryEntryKey) {
    let entry = match editor {
        AddEditor::Stdin => entryinput::read_from_stdin(),
        AddEditor::Environment => {
            let old_text = diary.text_for_entry(&key);
            entryinput::read_from_editor(&old_text)
        }
    };
    match entry {
        Ok(e) if !e.is_empty() => {
            diary.replace_entry(&e, key);
        }
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to read entry: {}", e);
            process::exit(1)
        }
    }
}

enum AddEditor {
    Environment,
    Stdin,
}

fn tags_with_args(diary: &CLIDiary, tags_matches: &clap::ArgMatches) {
    if let Some(tags_values) = tags_matches.values_of(args::tags::SEARCH) {
        let tags: Vec<&str> = tags_values.collect();
        search_tags(diary, &tags)
    } else if tags_matches.is_present(args::tags::REINDEX) {
        reindex(diary)
    }
}

fn search_tags(diary: &CLIDiary, tags: &[&str]) {
    let keys = diary.search_tags(tags);
    let entry_list = make_entry_list(&keys, ListOption::Plain, KeyOrdering::LatestFirst);
    for entry in entry_list {
        println!("{}", entry);
    }
}

fn reindex(diary: &CLIDiary) {
    diary.reindex()
}

mod args {
    pub mod opts {
        pub static NAME: &str = "name";
        pub static PATH: &str = "path";
    }

    pub mod add {
        pub static SUBCOMMAND: &str = "add";
        pub static STDIN: &str = "stdin";
        pub static DATE: &str = "date";
    }

    pub mod edit {
        pub static SUBCOMMAND: &str = "edit";
        pub static STDIN: &str = "stdin";
        pub static DATE: &str = "date";
    }

    pub mod list {
        pub static SUBCOMMAND: &str = "list";
        pub static ENUM: &str = "enumerate";
        pub static ENUM_REVERSE: &str = "enumerate-reverse";
        pub static SORT_REVERSE: &str = "sort-latest-first";
    }

    pub mod show {
        pub static SUBCOMMAND: &str = "show";
        pub static DATE: &str = "date";
        pub static NUMBER: &str = "number";
        pub static NUMBER_REVERSE: &str = "number-reverse";
    }

    pub mod tags {
        pub static SUBCOMMAND: &str = "tags";
        pub static SEARCH: &str = "search";
        pub static REINDEX: &str = "reindex";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(&(vec![]), ListOption::Plain, KeyOrdering::EarliestFirst)
        );
    }

    #[test]
    fn enumerated_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(&(vec![]), ListOption::Enumerate, KeyOrdering::EarliestFirst)
        );
    }

    #[test]
    fn reverse_enumerated_empty_entry_list_works() {
        assert_eq!(
            Vec::<String>::new(),
            make_entry_list(
                &(vec![]),
                ListOption::EnumerateReverse,
                KeyOrdering::EarliestFirst
            )
        );
    }

    #[test]
    fn plain_entry_list_works() {
        let dts1 = "2020-07-10 09:11 +0000";
        let dts2 = "2020-07-10 20:51 +0000";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(
            &(vec![k1, k2]),
            ListOption::Plain,
            KeyOrdering::EarliestFirst,
        );

        assert_eq!(vec![dts1, dts2], entry_list);
    }

    #[test]
    fn enumerated_entry_list_works() {
        let dts1 = "2020-07-10 09:11 +0000";
        let dts2 = "2020-07-10 20:51 +0000";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(
            &(vec![k1, k2]),
            ListOption::Enumerate,
            KeyOrdering::EarliestFirst,
        );

        assert_eq!(
            vec![format!("1 {}", dts1), format!("2 {}", dts2)],
            entry_list
        );
    }

    #[test]
    fn reverse_enumerated_entry_list_works() {
        let dts1 = "2020-07-10 09:11 +0000";
        let dts2 = "2020-07-10 20:51 +0000";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(
            &(vec![k1, k2]),
            ListOption::EnumerateReverse,
            KeyOrdering::EarliestFirst,
        );

        assert_eq!(
            vec![format!("2 {}", dts1), format!("1 {}", dts2)],
            entry_list
        );
    }

    #[test]
    fn reverse_enumerated_entry_list_latest_first() {
        let dts1 = "2020-07-10 09:11 +0000";
        let dts2 = "2020-07-10 20:51 +0000";
        let k1 = DiaryEntryKey::parse_from_string(dts1).expect("Parsing dts1 failed");
        let k2 = DiaryEntryKey::parse_from_string(dts2).expect("Parsing dts2 failed");
        let entry_list = make_entry_list(
            &(vec![k1, k2]),
            ListOption::EnumerateReverse,
            KeyOrdering::LatestFirst,
        );

        assert_eq!(
            vec![format!("1 {}", dts2), format!("2 {}", dts1)],
            entry_list
        );
    }
}
