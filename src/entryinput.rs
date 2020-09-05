extern crate tempfile;

use std::env;
use std::fs;
use std::io::Write;
use std::io::{self, Read};
use std::process::{self, Command};

pub fn read_from_editor(text: &str) -> io::Result<String> {
    open_external_editor(&(name_from_env()), text)
}

fn name_from_env() -> String {
    env::var("VISUAL")
        .ok()
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| {
            eprintln!("Couldn't find VISUAL or EDITOR in environment");
            process::exit(1)
        })
}

fn open_external_editor(editor: &str, text: &str) -> io::Result<String> {
    let mut inputfile = tempfile::NamedTempFile::new()?;
    if !text.is_empty() {
        inputfile.write_all(text.as_bytes())?;
    }
    let path = inputfile.into_temp_path();
    Command::new(editor).arg(&path).status()?;
    let content = fs::read_to_string(&path)?;
    path.close()?;

    Ok(content)
}

pub fn read_from_stdin() -> io::Result<String> {
    let mut content = String::new();
    io::stdin().read_to_string(&mut content)?;
    Ok(content)
}
