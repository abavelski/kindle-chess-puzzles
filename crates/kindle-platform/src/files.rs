//! Atomic replacement of opaque bytes; callers own validation and overwrite protection.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("could not create file directory", parent, error))?;

    let (temp_path, mut temp_file) = create_temp_file(parent, path)?;
    if let Err(error) = temp_file.write_all(bytes) {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(io_error(
            "could not write temporary file",
            &temp_path,
            error,
        ));
    }
    if let Err(error) = temp_file.sync_all() {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(io_error("could not sync temporary file", &temp_path, error));
    }
    drop(temp_file);

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(io_error("could not atomically replace file", path, error));
    }

    sync_directory(parent)?;
    Ok(())
}

fn create_temp_file(parent: &Path, target: &Path) -> Result<(PathBuf, File), String> {
    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.json");

    for _ in 0..128 {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_path = parent.join(format!(".{target_name}.tmp-{}-{sequence}", process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(io_error(
                    "could not create temporary file",
                    &temp_path,
                    error,
                ));
            }
        }
    }

    Err(format!(
        "could not allocate a temporary file in {}",
        parent.display()
    ))
}

pub fn sync_directory(path: &Path) -> Result<(), String> {
    let directory = File::open(path)
        .map_err(|error| io_error("could not open directory for sync", path, error))?;
    directory
        .sync_all()
        .map_err(|error| io_error("could not sync directory", path, error))
}

fn io_error(context: &str, path: &Path, error: io::Error) -> String {
    format!("{context} {}: {error}", path.display())
}
