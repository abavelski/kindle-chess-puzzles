use chess_core::{puzzle_app::PuzzleAction as P, review_app::ReviewAction as R};
use kindle_app::{
    products::{review_paths, PuzzleRuntime, ReviewRuntime},
    runtime::Application,
    StoragePaths,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Root(PathBuf);
impl Root {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "split-products-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn standalone_startup_loads_only_its_product_and_flushes_separate_state() {
    let root = Root::new();
    let puzzle_root = root.0.join("puzzles-app");
    let review_root = root.0.join("review-app");
    fs::create_dir_all(review_root.join("games")).unwrap();
    fs::write(
        review_root.join("games/games.json"),
        include_bytes!("../../../tests/fixtures/game-review/valid-standard.json"),
    )
    .unwrap();
    let puzzle_paths = StoragePaths::new(
        puzzle_root.join("puzzles"),
        puzzle_root.join("state/progress.json"),
    );
    // Puzzles must not discover or parse even a broken review resume/library.
    fs::create_dir_all(&puzzle_paths.review_dir).unwrap();
    fs::write(puzzle_paths.review_dir.join("games.json"), b"broken games").unwrap();
    fs::create_dir_all(puzzle_root.join("state")).unwrap();
    fs::write(
        &puzzle_paths.review_resume_file,
        b"future/corrupt review resume",
    )
    .unwrap();
    let mut puzzles = PuzzleRuntime::open(puzzle_paths.clone()).unwrap();
    assert!(puzzles.state().transient_message().is_none());
    let puzzle_bytes = fs::read(&puzzle_paths.progress_file).unwrap();
    let review_paths = review_paths(&review_root);
    fs::create_dir_all(review_root.join("state")).unwrap();
    fs::write(
        &review_paths.progress_file,
        b"future/corrupt puzzle progress",
    )
    .unwrap();
    let mut review = ReviewRuntime::open(review_paths.clone()).unwrap();
    assert!(review.state().transient_message().is_none());
    review.dispatch(R::ReviewNext);
    review.flush();
    assert!(review_paths.review_resume_file.is_file());
    assert!(!review_paths.puzzle_dir.exists());
    assert_eq!(
        fs::read(&review_paths.progress_file).unwrap(),
        b"future/corrupt puzzle progress"
    );
    puzzles.dispatch(P::NextPuzzle);
    puzzles.flush();
    assert_ne!(fs::read(&puzzle_paths.progress_file).unwrap(), puzzle_bytes);
    assert_eq!(
        fs::read(&puzzle_paths.review_resume_file).unwrap(),
        b"future/corrupt review resume"
    );
    assert_eq!(
        fs::read(puzzle_paths.review_dir.join("games.json")).unwrap(),
        b"broken games"
    );
}
