# Dear Diary: A simple CLI diary app

Extremely work in progress.

Dear Diary is a simple command line tool for managing diaries (i.e. things where you, the user, write things down as they happen). They are stored in your local file system as plain text files.

## Usage

`ddiary` creates a diaries in either a user-selected or a platform specific default location (`$HOME/Application Support/fi.juripakaste.Dear-Diary` on macOS). It supports multiple diaries per diary location, with default called `default`. To use a custom location, use the `--path` option. To use a different name for your diary, use the `--name` option.

See `ddiary --help` for a list of all command line options.

### Adding a diary entry

To add a diary entry, run `ddiary` without any extra parameters (other than possibly `--name` or `--path`, see above), or use the `add` subcommand with `ddiary add`. It will try to find a suitable editor to use in the `VISUAL` or `EDITOR` environment variables and if both are undefined, it will try to read the new entry from the standard input.

You can force standard input with `ddiary add --stdin`, and you can override the current date with the `--date` option: `ddiary add --date "2020-07-01 10:00 +0000"`.

### Listing diary entries

Running `ddiary list` produces a list of diary entries, one entry per line. Each diary entry is identified by a date and time (unique identification happens with the precision of one minute.) You can ask `list` to attach numbers to each entry, and you can ask them to be listed in reverse order instead of earliest one first.

### Displaying a diary entry

The subcommand `show`, i.e. `ddiary show`, will display one entry. You can select the entry with a date, as displayed in `ddiary list`, or with a number, as shown in `ddiary show -e` or `ddiary show -E`. Without any extra parameters `show` will display the latest entry.

### Tagging diary entries

Diary entries can contain tags. A single-word tag is a hash mark (`#`) followed by one
or more letters or digits. Multiword tags are surrounded by `#(` and `)#`, and you 
can use any number of hash marks as long as the start and end markers have the same number.

Example:

> A #diary #entry with ##(many tags)##

You can search tags with `ddiary tags -s tag1 tag2`. If your index goes bad, `ddiary tags -I`
will recreate it.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
