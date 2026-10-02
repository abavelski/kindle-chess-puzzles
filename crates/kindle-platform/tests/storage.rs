use chess_core::Progress;
use kindle_platform::{KindleStorage, ProgressStore, StoragePaths};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BUNDLED: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(name: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kindle-chess-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test temp root");
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

fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, bytes).expect("write fixture");
}

#[test]
fn discovery_filters_sorts_and_uses_filename_when_title_is_blank() {
    let root = TempRoot::new("discovery");
    let paths = root.paths();
    write(
        &paths.puzzle_dir.join("puzzles-a.json"),
        br#"{
          "version":1,
          "title":"   ",
          "puzzles":[{
            "id":"a",
            "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
            "solution":["g6g7"]
          }]
        }"#,
    );
    write(&paths.puzzle_dir.join("puzzles-z.json"), PUZZLES);
    write(
        &paths.puzzle_dir.join("puzzles-broken.json"),
        b"{ definitely not json",
    );
    write(&paths.puzzle_dir.join("notes.json"), b"{}");
    fs::create_dir_all(paths.puzzle_dir.join("puzzles-dir.json"))
        .expect("create ignored directory");

    let storage = KindleStorage::new(paths);
    let discovered = storage
        .discover_collections()
        .expect("discover collections");

    assert_eq!(
        discovered
            .iter()
            .map(|entry| entry.filename.as_str())
            .collect::<Vec<_>>(),
        ["puzzles-a.json", "puzzles-broken.json", "puzzles-z.json"]
    );
    assert_eq!(discovered[0].label, "puzzles-a.json");
    assert!(discovered[0].is_valid());
    assert_eq!(discovered[1].label, "puzzles-broken.json");
    assert!(discovered[1].error.is_some());
    assert_eq!(discovered[2].label, "Lichess sample puzzles");
}

#[test]
fn no_collection_installs_bundled_examples_as_puzzles_json() {
    let root = TempRoot::new("install-default");
    let paths = root.paths();
    write(&root.path().join("unrelated.txt"), b"leave me alone");

    let storage = KindleStorage::new(paths.clone());
    let discovered = storage
        .prepare_collections(BUNDLED)
        .expect("install bundled default");

    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].filename, "puzzles.json");
    assert!(discovered[0].is_valid());
    assert_eq!(
        fs::read(paths.puzzle_dir.join("puzzles.json")).expect("read installed collection"),
        BUNDLED
    );
    assert_eq!(
        fs::read(root.path().join("unrelated.txt")).expect("read unrelated file"),
        b"leave me alone"
    );
}

#[test]
fn invalid_only_discovery_preserves_source_and_does_not_install_over_it() {
    let root = TempRoot::new("invalid-only");
    let paths = root.paths();
    let broken_path = paths.puzzle_dir.join("puzzles-broken.json");
    let broken = b"{ broken user data";
    write(&broken_path, broken);

    let storage = KindleStorage::new(paths.clone());
    let discovered = storage
        .prepare_collections(BUNDLED)
        .expect("invalid files are discoverable");

    assert_eq!(discovered.len(), 1);
    assert!(!discovered[0].is_valid());
    assert_eq!(
        fs::read(&broken_path).expect("read preserved broken file"),
        broken
    );
    assert!(!paths.puzzle_dir.join("puzzles.json").exists());
}

#[test]
fn progress_round_trips_through_atomic_store_and_restores_ids() {
    let root = TempRoot::new("progress-round-trip");
    let paths = root.paths();
    let (mut store, load) = ProgressStore::open(paths.progress_file.clone());
    assert!(!store.protected());
    assert!(load.warning.is_none());

    let mut progress = load.progress;
    progress.set_active_file("puzzles-endgames.json");
    progress.remember_puzzle("puzzles-endgames.json", "end-2");
    progress.mark_solved("puzzles-endgames.json", "end-1");
    assert!(store.save_latest(&progress).expect("save progress"));
    assert!(!store.dirty());

    let (reopened, loaded) = ProgressStore::open(paths.progress_file);
    assert!(!reopened.protected());
    assert_eq!(loaded.progress, progress);
    assert!(loaded.warning.is_none());
}

#[test]
fn progress_write_failure_keeps_dirty_state_and_later_retry_saves_latest_memory() {
    let root = TempRoot::new("dirty-retry");
    let paths = root.paths();
    let (mut store, load) = ProgressStore::open(paths.progress_file.clone());
    let mut progress = load.progress;
    progress.set_active_file("puzzles.json");
    progress.remember_puzzle("puzzles.json", "one-move");

    fs::create_dir_all(&paths.progress_file).expect("block destination with directory");
    assert!(store.save_latest(&progress).is_err());
    assert!(store.dirty());

    progress.mark_solved("puzzles.json", "one-move");
    fs::remove_dir_all(&paths.progress_file).expect("remove failure injection");
    assert!(store
        .retry_if_dirty(&progress)
        .expect("retry dirty progress"));
    assert!(!store.dirty());

    let bytes = fs::read(&paths.progress_file).expect("read retried progress");
    let saved = Progress::parse(&bytes).expect("parse retried progress");
    assert!(saved.is_solved("puzzles.json", "one-move"));
}

#[test]
fn corrupt_and_future_progress_are_protected_from_overwrite() {
    for (name, bytes) in [
        ("corrupt", b"not json".as_slice()),
        (
            "future",
            br#"{"version":2,"active_file":"puzzles.json","files":{}}"#.as_slice(),
        ),
    ] {
        let root = TempRoot::new(name);
        let paths = root.paths();
        write(&paths.progress_file, bytes);
        let original = fs::read(&paths.progress_file).expect("read original");

        let (mut store, load) = ProgressStore::open(paths.progress_file.clone());
        assert!(store.protected());
        assert!(load.warning.is_some());

        let mut newest = load.progress;
        newest.set_active_file("puzzles.json");
        assert!(store.save_latest(&newest).is_err());
        assert!(store.dirty());
        assert_eq!(
            fs::read(&paths.progress_file).expect("read protected progress"),
            original
        );
    }
}
