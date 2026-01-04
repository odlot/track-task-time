use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use directories::ProjectDirs;

use crate::crypto::{decrypt_store, encrypt_store};
use crate::model::Store;

const BACKUP_COUNT: usize = 3;

pub struct BackupEntry {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

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

    if !is_backup_path(path) {
        rotate_backups(path)?;
    }

    let payload = encrypt_store(store, passphrase)?;
    write_secure(path, payload.as_bytes())
}

pub fn list_backups(path: &Path) -> Vec<BackupEntry> {
    let mut entries = Vec::new();
    for index in 1..=BACKUP_COUNT {
        let backup_path = backup_path(path, index);
        if let Ok(metadata) = fs::metadata(&backup_path) {
            entries.push(BackupEntry {
                path: backup_path,
                modified: metadata.modified().ok(),
                size: metadata.len(),
            });
        }
    }
    entries
}

fn rotate_backups(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let oldest = backup_path(path, BACKUP_COUNT);
    if oldest.exists() {
        fs::remove_file(&oldest).map_err(|err| err.to_string())?;
    }

    for index in (1..BACKUP_COUNT).rev() {
        let src = backup_path(path, index);
        if src.exists() {
            let dest = backup_path(path, index + 1);
            fs::rename(&src, &dest).map_err(|err| err.to_string())?;
        }
    }

    let backup = backup_path(path, 1);
    fs::copy(path, &backup).map_err(|err| err.to_string())?;
    set_permissions_secure(&backup)?;
    Ok(())
}

fn backup_path(path: &Path, index: usize) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("ttt.json");
    let backup_name = format!("{}.bak{}", name, index);
    let mut backup_path = path.to_path_buf();
    backup_path.set_file_name(backup_name);
    backup_path
}

fn is_backup_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|name| name.contains(".bak"))
        .unwrap_or(false)
}

fn write_secure(path: &Path, payload: &[u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|err| err.to_string())?;
        use std::io::Write;
        file.write_all(payload).map_err(|err| err.to_string())?;
        set_permissions_secure(path)?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        fs::write(path, payload).map_err(|err| err.to_string())
    }
}

fn set_permissions_secure(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}
