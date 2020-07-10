extern crate tempfile;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::Command;

enum Editor {
    External(String),
    StandardInput,
}

pub fn read_entry() -> io::Result<String> {
    match editor_from_env() {
        Editor::External(name) => open_external_editor(&name),
        Editor::StandardInput => read_from_stdin(),
    }
}

fn name_from_env() -> Option<String> {
    env::var("VISUAL").ok().or_else(|| env::var("EDITOR").ok())
}

fn editor_from_env() -> Editor {
    name_from_env()
        .map(Editor::External)
        .unwrap_or(Editor::StandardInput)
}

fn open_external_editor(editor: &str) -> io::Result<String> {
    let inputfile = tempfile::NamedTempFile::new()?;
    let path = inputfile.into_temp_path();
    Command::new(editor).arg(&path).status()?;
    let content = fs::read_to_string(&path)?;
    path.close()?;

    Ok(content)
}

fn read_from_stdin() -> io::Result<String> {
    let mut content = String::new();
    io::stdin().read_to_string(&mut content)?;
    Ok(content)
}
