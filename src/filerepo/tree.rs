use std::path::{Path, PathBuf};

use std::fmt;
use std::fs;
use std::io;
use std::string;

use chrono::prelude::*;

#[derive(Debug)]
pub struct Tree {
    root: PathBuf,
}

pub type FileRepoResult<T> = Result<T, FileRepoError>;

impl Tree {
    pub fn new(root: &Path) -> FileRepoResult<Tree> {
        if !root.is_dir() {
            FileRepoResult::Err(FileRepoError::BadPathError(root.to_path_buf()))
        } else {
            FileRepoResult::Ok(Tree {
                root: root.to_path_buf(),
            })
        }
    }

    pub fn list(&self) -> FileRepoResult<Vec<DateTime<Utc>>> {
        collect_dates(&self.root)
    }

    pub fn get_text(&self, dt: &DateTime<Utc>) -> FileRepoResult<String> {
        get_text(&self.root, dt)
    }
}

#[derive(Debug)]
pub enum FileRepoError {
    BadPathError(PathBuf),
    IoError(io::Error),
    NameParseError(String, chrono::ParseError),
    EntryContentDecodingError(string::FromUtf8Error),
}

impl fmt::Display for FileRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileRepoError::BadPathError(p) => write!(f, "Not a directory: {}", p.to_str().unwrap_or("(no path)")),
            FileRepoError::IoError(e) => { 
                write!(f, "IO Error: ")?;
                e.fmt(f)
            },
            FileRepoError::NameParseError(name, e) => 
                write!(f, "Date parse error with name {}: {}", name, e),
            FileRepoError::EntryContentDecodingError(e) =>
                write!(f, "Error decoding diary entry content: {}", e),
        }
    }
}

impl From<io::Error> for FileRepoError {
    fn from(error: io::Error) -> Self {
        FileRepoError::IoError(error)
    }
}

fn get_text(dir: &Path, dt: &DateTime<Utc>) -> FileRepoResult<String> {
    let path = file_path(dt);
    let data = fs::read(dir.join(path))?;
    match String::from_utf8(data) {
        Ok(s) => FileRepoResult::Ok(s),
        Err(e) => FileRepoResult::Err(FileRepoError::EntryContentDecodingError(e))
    }
}

fn file_path(dt: &DateTime<Utc>) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(format!("{:04}", dt.year()));
    path.push(format!("{:02}", dt.month()));
    path.push(dt.format(FILE_NAME_FORMAT).to_string());
    path
}

fn collect_dates(dir: &Path) -> FileRepoResult<Vec<DateTime<Utc>>> {
    // collect_files returns a Result. So match on it and if it's Ok,
    // then iterate the contained name, parse them, translate parse errors into
    // FileRepoErrors and then with collect change the Vec of Results into
    // a Result of Vec where an Err in one of the original Results renders the
    // total Result into an Err.

    collect_files(dir).map(|names| {
        names
            .iter()
            .filter_map(|name| Utc.datetime_from_str(name, FILE_NAME_FORMAT).ok())
            // .map(|name| Utc.datetime_from_str(name, FILE_NAME_FORMAT).map_err(|e| FileRepoError::NameParseError(String::from(name), e)))
            .collect()  // ::<FileRepoResult<Vec<DateTime<Utc>>>>(),
    })


    // match collect_files(dir) {
    //     Ok(names) => names
    //         .iter()
    //         .filter_map(|name| Utc.datetime_from_str(name, FILE_NAME_FORMAT).ok())
    //         // .map(|name| Utc.datetime_from_str(name, FILE_NAME_FORMAT).map_err(|e| FileRepoError::NameParseError(String::from(name), e)))
    //         .collect(),  // ::<FileRepoResult<Vec<DateTime<Utc>>>>(),
    //     Err(e) => Err(e),
    // }
}

fn collect_files(dir: &Path) -> FileRepoResult<Vec<String>> {
    let mut files: Vec<String> = Vec::new();
    let visitor = &mut |fp: &Path| -> () {
        let parent1 = match fp.parent() {
            Some(p) => p,
            None => return
        };
        let parent1_name = match parent1.file_name().and_then(|s| s.to_str()) {
            Some(p) => p,
            None => return
        };
        let parent2_name = match parent1.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) {
            Some(p) => p,
            None => return
        };
        // let fpstr = match fp.to_str() {
        //     Some(p) => p,
        //     None => return
        // };
        let file_name = match fp.file_name().and_then(|s| s.to_str()) {
            Some(p) => p,
            None => return
        };
        // if parent2_name == file_name.chars().take(4).collect() && parent1_name == file_name.chars().skip(4).take(2).collect() {
        let file_start1: String = file_name.chars().take(4).collect();
        let file_start2: String = file_name.chars().skip(4).take(2).collect();

        if parent2_name == file_start1 && parent1_name == file_start2 {
            let path_str = String::from(file_name);
            files.push(path_str);
        }
    };
    visit_dirs(dir, visitor)?;
    Ok(files)
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    }
    Ok(())
}

static FILE_NAME_FORMAT: &str = "%Y%m%dT%H%M";
