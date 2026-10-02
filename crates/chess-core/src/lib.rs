//! Platform-neutral chess puzzle compatibility core.
//!
//! This crate deliberately contains no Kindle, FBInk, Linux input, or storage
//! path logic. It models only the reusable board, FEN/UCI, puzzle collection,
//! collection-name, and progress behavior shared with the reference app.

#![forbid(unsafe_code)]

pub mod board;
pub mod collection;
pub mod fen;
pub mod progress;
pub mod puzzle;
pub mod uci;

pub use board::{Board, Color, Piece, PieceKind, TapResult};
pub use collection::{is_puzzle_collection_filename, sorted_puzzle_collection_filenames};
pub use fen::{parse_fen, FenError, FenPosition};
pub use progress::{FileProgress, Progress, PROGRESS_VERSION};
pub use puzzle::{parse_puzzle_file, Difficulty, Puzzle, PuzzleCollection, MAX_PUZZLE_FILE_BYTES};
pub use uci::{algebraic_to_square, parse_uci_move, square_to_algebraic, UciError, UciMove};
