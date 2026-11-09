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
    let mut paths = review_paths(&root);
    for (name, target) in [
        ("KINDLE_GAME_REVIEW_GAME_DIR", &mut paths.review_dir),
        ("KINDLE_GAME_REVIEW_SETTINGS_FILE", &mut paths.settings_file),
        (
            "KINDLE_GAME_REVIEW_RESUME_FILE",
            &mut paths.review_resume_file,
        ),
    ] {
        if let Some(value) = std::env::var_os(name) {
            *target = value.into();
        }
    }
    crate::runtime::run(ReviewRuntime::open(paths)?, start)
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

mod play;
pub use play::PlayRuntime;

pub fn run_play() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let root = std::env::var_os("KINDLE_CHESS_PLAY_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/mnt/us/kindle-chess-play".into());
    crate::runtime::run(PlayRuntime::open(root), start)
}
