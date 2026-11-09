//! Product repositories, board session orchestration and Kindle composition policy.
#![forbid(unsafe_code)]
pub mod chessnut;
mod refresh;
mod storage;
pub use refresh::clean_regions_for_board_change;
pub use storage::{
    is_review_collection_filename, DiscoveredCollection, DiscoveredReviewLibrary, KindleStorage,
    ProgressLoad, ProgressStore, ReviewResumeLoad, ReviewResumeStore, SettingsLoad, SettingsStore,
    StorageError, StoragePaths, DEFAULT_PROGRESS_FILE, DEFAULT_PUZZLE_DIR, DEFAULT_REVIEW_DIR,
    DEFAULT_SETTINGS_FILE, PROGRESS_FILE_ENV, PUZZLE_DIR_ENV, REVIEW_DIR_ENV,
    REVIEW_RESUME_FILE_ENV, SETTINGS_FILE_ENV,
};

pub mod runtime;

pub mod products;
pub mod startup;

pub mod play_http;
