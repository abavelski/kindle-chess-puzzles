use chess_core::Settings;
use kindle_platform::{SettingsStore, StoragePaths};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(name: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kindle-chess-settings-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn paths(&self) -> StoragePaths {
        StoragePaths::new(
            self.0.join("puzzles"),
            self.0.join("state").join("progress.json"),
        )
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn storage_paths_keep_settings_beside_progress_by_default() {
    let root = TempRoot::new("paths");
    let paths = root.paths();
    assert_eq!(
        paths.settings_file,
        root.path().join("state").join("settings.json")
    );
}

#[test]
fn settings_round_trip_through_atomic_store() {
    let root = TempRoot::new("round-trip");
    let paths = root.paths();
    let (mut store, load) = SettingsStore::open(paths.settings_file.clone());
    assert_eq!(load.settings, Settings::default());
    assert!(load.warning.is_none());

    let mut settings = load.settings;
    settings.set_show_free_mode_button(false);
    settings.set_show_notes_button(false);
    assert!(store.save_latest(&settings).expect("save settings"));

    let (reopened, loaded) = SettingsStore::open(paths.settings_file);
    assert!(!reopened.protected());
    assert_eq!(loaded.settings, settings);
}

#[test]
fn malformed_settings_are_protected_from_overwrite() {
    let root = TempRoot::new("protected");
    let paths = root.paths();
    fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
    fs::write(&paths.settings_file, b"not json").unwrap();
    let original = fs::read(&paths.settings_file).unwrap();

    let (mut store, load) = SettingsStore::open(paths.settings_file.clone());
    assert!(store.protected());
    assert!(load.warning.is_some());
    assert!(store.save_latest(&Settings::default()).is_err());
    assert_eq!(fs::read(&paths.settings_file).unwrap(), original);
}
