use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::crypto::{decrypt_store, encrypt_store};
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

pub fn load_store(path: &Path, passphrase: &str) -> Result<Store, String> {
    if !path.exists() {
        return Ok(Store {
            version: 1,
            tasks: Vec::new(),
        });
    }

    let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
    decrypt_store(&contents, passphrase)
}

pub fn save_store(path: &Path, store: &Store, passphrase: &str) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let payload = encrypt_store(store, passphrase)?;
    write_secure(path, payload.as_bytes())
}

fn write_secure(path: &Path, payload: &[u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|err| err.to_string())?;
        use std::io::Write;
        file.write_all(payload).map_err(|err| err.to_string())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        fs::write(path, payload).map_err(|err| err.to_string())
    }
}
