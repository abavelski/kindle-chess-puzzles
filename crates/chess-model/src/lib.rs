//! Reusable chess positions, coordinates and authored analysis trees.
#![forbid(unsafe_code)]
pub mod analysis;
pub mod board;
pub mod fen;
pub mod uci;
pub use analysis::{
    AnalysisMove, AnalysisNode, AnalysisNodeIndex, AnalysisRole, AnalysisTextSpan, AnalysisTree,
    Nag,
};
pub use board::{Board, Color, Piece, PieceKind, TapResult};
pub use fen::{parse_fen, FenError, FenPosition};
pub use uci::{algebraic_to_square, parse_uci_move, square_to_algebraic, UciError, UciMove};
