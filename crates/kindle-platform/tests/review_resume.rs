use chess_core::*;
use kindle_platform::{ReviewResumeStore, StoragePaths};
use std::fs;

fn resume() -> ReviewResume {
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let game = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap()
    .games
    .remove(0);
    app.set_review_games(vec![ReviewGameEntry::new("games.json", game)]);
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ReviewNext);
    app.review_resume().unwrap()
}

#[test]
fn resume_store_retries_latest_state_and_round_trips_separately() {
    let root = std::env::temp_dir().join(format!("review-resume-store-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let paths = StoragePaths::new(root.join("puzzles"), root.join("state/progress.json"));
    assert_eq!(
        paths.review_resume_file,
        root.join("state/review-resume.json")
    );
    fs::write(root.join("state"), b"blocked").unwrap();
    let (_, load) = ReviewResumeStore::open(paths.review_resume_file.clone());
    // Use a readable missing path for a new store, then block the parent before saving.
    fs::remove_file(root.join("state")).unwrap();
    let (fresh, fresh_load) = ReviewResumeStore::open(paths.review_resume_file.clone());
    assert!(fresh_load.resume.is_none());
    let mut store = fresh;
    assert!(load.warning.is_some());
    fs::write(root.join("state"), b"blocked").unwrap();
    let mut latest = resume();
    store.observe(latest.clone());
    assert!(store.retry_if_dirty().is_err());
    assert!(store.dirty());
    latest.flipped = !latest.flipped;
    store.observe(latest.clone());
    fs::remove_file(root.join("state")).unwrap();
    assert!(store.retry_if_dirty().unwrap());
    assert!(!store.dirty());
    let (_, load) = ReviewResumeStore::open(paths.review_resume_file);
    assert_eq!(load.resume, Some(latest));
    assert!(!paths.progress_file.exists());
    assert!(!paths.settings_file.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_future_and_unreadable_resume_files_are_preserved() {
    let root = std::env::temp_dir().join(format!("review-resume-protected-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("resume.json");
    for bytes in [b"not json".as_slice(), br#"{"version":99}"#] {
        fs::write(&path, bytes).unwrap();
        let (mut store, load) = ReviewResumeStore::open(&path);
        assert!(load.warning.is_some());
        store.observe(resume());
        assert!(store.retry_if_dirty().is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let (mut store, load) = ReviewResumeStore::open(&path);
    assert!(load.warning.is_some());
    store.observe(resume());
    assert!(store.retry_if_dirty().is_err());
    assert!(path.is_dir());
    fs::remove_dir_all(root).unwrap();
}
