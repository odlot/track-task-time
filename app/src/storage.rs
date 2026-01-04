use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::model::Store;

pub fn data_file_path(custom: Option<PathBuf>) -> PathBuf {
    if let Some(path) = custom {
        return path;
    }

    if let Some(dirs) = ProjectDirs::from("com", "ttt", "ttt") {
        return dirs.data_dir().join("ttt.json");
    }

    PathBuf::from("ttt.json")
}

pub fn load_store(path: &Path) -> Result<Store, String> {
    if !path.exists() {
        return Ok(Store {
            version: 1,
            tasks: Vec::new(),
        });
    }

    let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let store: Store = serde_json::from_str(&contents).map_err(|err| err.to_string())?;
    Ok(store)
}

pub fn save_store(path: &Path, store: &Store) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let payload = serde_json::to_string_pretty(store).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}
