use chess_core::{
    review_app::{ReviewAction as A, ReviewApp},
    *,
};
use kindle_app::{
    products::{review_paths, ReviewRuntime},
    runtime::Application,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
    legacy: PathBuf,
    review: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "split-migration-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let legacy = root.join("legacy");
        let review = root.join("review");
        fs::create_dir_all(legacy.join("games")).unwrap();
        fs::create_dir_all(legacy.join("state")).unwrap();
        let bytes = include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
        fs::write(legacy.join("games/games.json"), bytes).unwrap();
        fs::write(
            legacy.join("state/progress.json"),
            b"future puzzle progress: preserve exactly",
        )
        .unwrap();
        fs::write(
            legacy.join("state/settings.json"),
            include_bytes!("../../../tests/fixtures/review-migration/legacy-settings.json"),
        )
        .unwrap();
        let game = parse_review_file(bytes).unwrap().games.remove(0);
        let mut app = ReviewApp::new(
            vec![ReviewGameEntry::new("games.json", game)],
            WorkspaceSettings::default(),
        );
        app.dispatch(A::ReviewNext);
        app.dispatch(A::ReviewFlip);
        let saved = app.review_resume().unwrap();
        fs::write(
            legacy.join("state/review-resume.json"),
            saved.to_bytes().unwrap(),
        )
        .unwrap();
        Self {
            root,
            legacy,
            review,
        }
    }
    fn legacy_bytes(&self) -> Vec<Vec<u8>> {
        [
            "games/games.json",
            "state/progress.json",
            "state/settings.json",
            "state/review-resume.json",
        ]
        .iter()
        .map(|path| fs::read(self.legacy.join(path)).unwrap())
        .collect()
    }
    fn open(&self) -> ReviewRuntime {
        ReviewRuntime::open(review_paths(&self.review)).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn missing_review_inputs_ignore_populated_puzzles_and_relaunch_with_defaults() {
    let f = Fixture::new();
    let before = f.legacy_bytes();
    let mut app = f.open();
    assert!(app.state().review_games().is_empty());
    assert!(app.state().active_review_game().is_none());
    assert_eq!(app.state().review_resume(), None);
    assert!(!app.state().settings().small_board());
    app.flush();
    assert!(!f.review.exists());
    app.dispatch(A::OpenSettings);
    app.dispatch(A::ToggleBoardSizeSetting);
    app.dispatch(A::CloseSettings);
    let reopened = f.open();
    assert!(reopened.state().settings().small_board());
    assert!(reopened.state().review_games().is_empty());
    assert_eq!(reopened.state().review_resume(), None);
    assert!(!f.review.join("puzzles").exists());
    assert!(!f.review.join("state/progress.json").exists());
    assert!(!f.review.join("state/review-resume.json").exists());
    assert_eq!(f.legacy_bytes(), before);
}

#[test]
fn malformed_or_future_owned_state_stays_protected_without_fallback() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/review-migration/interrupted-state.bin").as_slice(),
        include_bytes!("../../../tests/fixtures/review-migration/future-state.json").as_slice(),
    ] {
        let f = Fixture::new();
        let before = f.legacy_bytes();
        fs::create_dir_all(f.review.join("state")).unwrap();
        for file in ["settings.json", "review-resume.json"] {
            fs::write(f.review.join("state").join(file), bytes).unwrap();
        }
        let mut app = f.open();
        assert!(!app.state().settings().small_board());
        app.dispatch(A::ReviewNext);
        app.dispatch(A::OpenSettings);
        app.dispatch(A::ToggleBoardSizeSetting);
        app.flush();
        for file in ["settings.json", "review-resume.json"] {
            let path = f.review.join("state").join(file);
            assert_eq!(fs::read(&path).unwrap(), bytes);
            // Explicit rollback of only these disposable new-root fixtures.
            fs::remove_file(path).unwrap();
        }
        let restored = f.open();
        assert_eq!(restored.state().review_resume(), None);
        assert!(!restored.state().settings().small_board());
        assert_eq!(f.legacy_bytes(), before);
    }
}

#[test]
fn owned_library_and_state_win_and_invalid_legacy_bytes_are_never_rewritten() {
    let f = Fixture::new();
    fs::create_dir_all(f.review.join("games")).unwrap();
    fs::create_dir_all(f.review.join("state")).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(f.legacy.join("games/games.json")).unwrap()).unwrap();
    value["games"][0]["id"] = "new-game".into();
    fs::write(
        f.review.join("games/games.json"),
        serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    fs::write(
        f.review.join("state/settings.json"),
        br#"{"version":1,"small_board":false}"#,
    )
    .unwrap();
    let game = parse_review_file(&serde_json::to_vec(&value).unwrap())
        .unwrap()
        .games
        .remove(0);
    let mut own = ReviewApp::new(
        vec![ReviewGameEntry::new("games.json", game)],
        WorkspaceSettings::default(),
    );
    own.dispatch(A::ReviewNext);
    let own_resume = own.review_resume().unwrap();
    fs::write(
        f.review.join("state/review-resume.json"),
        own_resume.to_bytes().unwrap(),
    )
    .unwrap();
    for file in ["settings.json", "review-resume.json"] {
        fs::write(
            f.legacy.join("state").join(file),
            b"future/corrupt legacy bytes",
        )
        .unwrap();
    }
    let before = f.legacy_bytes();
    let mut app = f.open();
    assert_eq!(
        app.state().active_review_game().unwrap().key().game_id(),
        "new-game"
    );
    assert!(!app.state().settings().small_board());
    assert_eq!(app.state().review_resume(), Some(own_resume));
    app.dispatch(A::ReviewNext);
    app.flush();
    assert_eq!(f.legacy_bytes(), before);
}

#[test]
fn fresh_install_and_traversal_keys_never_initialize_puzzles() {
    let f = Fixture::new();
    let absent = f.root.join("absent");
    let mut fresh = ReviewRuntime::open(review_paths(&absent)).unwrap();
    fresh.flush();
    assert!(!absent.exists());
    let storage = kindle_app::KindleStorage::new(review_paths(&f.legacy));
    for collection in ["../games.json", "/games.json", "games-../games.json"] {
        assert!(storage
            .load_review_game(&ReviewGameKey::new(collection, "id"))
            .is_err());
    }
}

#[test]
fn corrupt_owned_library_is_preserved_without_loading_puzzles() {
    let f = Fixture::new();
    let before = f.legacy_bytes();
    fs::create_dir_all(f.review.join("games")).unwrap();
    let path = f.review.join("games/games.json");
    fs::write(&path, b"corrupt owned games").unwrap();
    let mut app = f.open();
    assert!(app.state().review_games().is_empty());
    assert_eq!(app.state().review_file_errors().len(), 1);
    app.flush();
    assert_eq!(fs::read(path).unwrap(), b"corrupt owned games");
    assert_eq!(f.legacy_bytes(), before);
}

#[test]
fn invalid_legacy_state_stays_opaque_while_new_preferences_are_saved_separately() {
    let f = Fixture::new();
    for file in ["settings.json", "review-resume.json"] {
        fs::write(
            f.legacy.join("state").join(file),
            include_bytes!("../../../tests/fixtures/review-migration/future-state.json"),
        )
        .unwrap();
    }
    let before = f.legacy_bytes();
    let mut app = f.open();
    assert!(!app
        .state()
        .transient_message()
        .unwrap_or_default()
        .contains("Legacy"));
    app.dispatch(A::OpenSettings);
    app.dispatch(A::ToggleBoardSizeSetting);
    app.dispatch(A::CloseSettings);
    app.dispatch(A::ReviewNext);
    app.flush();
    assert!(f.review.join("state/settings.json").is_file());
    assert_eq!(f.legacy_bytes(), before);
}
