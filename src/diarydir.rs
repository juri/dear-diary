extern crate directories;

use directories::ProjectDirs;

use std::path::PathBuf;

pub fn default_dir() -> Option<PathBuf> {
    ProjectDirs::from("fi", "juripakaste", "Dear Diary").map(|pd| pd.data_dir().to_path_buf())
}
