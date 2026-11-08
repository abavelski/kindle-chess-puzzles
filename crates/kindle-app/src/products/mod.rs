//! Separate composition roots; only the device loop is shared.
mod puzzles;
mod review;
pub use puzzles::PuzzleRuntime;
pub use review::{review_paths, ReviewRuntime};

pub fn run_puzzles() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    crate::runtime::run(PuzzleRuntime::open(crate::StoragePaths::from_env())?, start)
}
pub fn run_review() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let root = std::env::var_os("KINDLE_GAME_REVIEW_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/mnt/us/kindle-game-review".into());
    crate::runtime::run(ReviewRuntime::open(review_paths(&root))?, start)
}

fn is_persistence_warning(message: &str) -> bool {
    [
        "Progress warning:",
        "Settings warning:",
        "Review resume warning:",
    ]
    .iter()
    .any(|prefix| message.starts_with(prefix))
}
