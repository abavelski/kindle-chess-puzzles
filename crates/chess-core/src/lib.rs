//! Platform-neutral chess puzzle and game-review core.
//!
//! This crate deliberately contains no Kindle, FBInk, Linux input, storage path,
//! or PGN parsing logic. It models reusable board, FEN/UCI, puzzle/review data,
//! application state, collection-name, settings, and progress behavior.

#![forbid(unsafe_code)]

pub mod analysis;
pub mod app;
pub mod board;
pub mod collection;
pub mod fen;
pub mod progress;
pub mod puzzle;
pub mod review;
pub mod review_resume;
pub mod review_state;
pub mod settings;
pub mod uci;

pub use analysis::{
    AnalysisMove, AnalysisNode, AnalysisNodeIndex, AnalysisRole, AnalysisTextSpan, AnalysisTree,
    Nag,
};
pub use app::{
    Action, ActiveCollection, AnalysisBrowserState, AppState, BoardMode, Effect, PendingPromotion,
    PromotionChoice, SolutionFeedback, COLLECTIONS_PER_PAGE,
};
pub use board::{Board, Color, Piece, PieceKind, TapResult};
pub use collection::{
    is_puzzle_collection_filename, sorted_puzzle_collection_filenames, CollectionEntry,
};
pub use fen::{parse_fen, FenError, FenPosition};
pub use progress::{FileProgress, Progress, PROGRESS_VERSION};
pub use puzzle::{
    parse_puzzle_file, Difficulty, Puzzle, PuzzleCollection, LEGACY_PUZZLE_FILE_WARNING_BYTES,
    MAX_PUZZLE_FILE_BYTES,
};
pub use review::{
    parse_review_file, ReviewCollection, ReviewGame, ReviewGameKey, ReviewMetadata, ReviewResult,
    MAX_REVIEW_FILE_BYTES,
};
pub use review_state::{
    ReviewFileError, ReviewGameEntry, ReviewState, Workspace, REVIEW_GAMES_PER_PAGE,
};
pub use settings::{Settings, WorkspaceSettings, SETTINGS_VERSION};
pub use uci::{algebraic_to_square, parse_uci_move, square_to_algebraic, UciError, UciMove};

pub use review_resume::ReviewResume;
