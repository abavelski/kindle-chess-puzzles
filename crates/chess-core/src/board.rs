//! Physical chessboard state without chess legality enforcement.

use crate::{fen::parse_fen, uci::UciMove, FenError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TapResult {
    NoChange,
    SelectionChanged,
    Moved {
        from: usize,
        to: usize,
    },
    Promotion {
        from: usize,
        to: usize,
        color: Color,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    squares: [Option<Piece>; 64],
    starting_squares: [Option<Piece>; 64],
    selected: Option<usize>,
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position()
    }
}

impl Board {
    pub fn starting_position() -> Self {
        let mut squares = [None; 64];
        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];

        for (file, kind) in back_rank.into_iter().enumerate() {
            squares[file] = Some(Piece {
                color: Color::Black,
                kind,
            });
            squares[8 + file] = Some(Piece {
                color: Color::Black,
                kind: PieceKind::Pawn,
            });
            squares[48 + file] = Some(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            });
            squares[56 + file] = Some(Piece {
                color: Color::White,
                kind,
            });
        }

        Self {
            squares,
            starting_squares: squares,
            selected: None,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let parsed = parse_fen(fen)?;
        let squares = parsed.into_squares();
        Ok(Self {
            squares,
            starting_squares: squares,
            selected: None,
        })
    }

    pub fn reset(&mut self) {
        self.squares = self.starting_squares;
        self.selected = None;
    }

    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn piece_at(&self, square: usize) -> Option<Piece> {
        self.squares.get(square).copied().flatten()
    }

    /// Move a piece directly without applying chess legality.
    ///
    /// Moving onto an occupied square replaces that piece. This is also the
    /// primitive used for stored puzzle replies that do not promote.
    pub fn move_piece(&mut self, from: usize, to: usize) -> bool {
        if from >= 64 || to >= 64 || from == to {
            return false;
        }
        let Some(piece) = self.squares[from].take() else {
            return false;
        };
        self.squares[to] = Some(piece);
        self.selected = None;
        true
    }

    /// Apply a validated stored UCI move directly to the physical board.
    pub fn apply_uci_move(&mut self, movement: UciMove) -> bool {
        if let Some(kind) = movement.promotion {
            self.promote_pawn(movement.from, movement.to, kind)
        } else {
            self.move_piece(movement.from, movement.to)
        }
    }

    /// Promote a pawn atomically without applying other chess legality.
    pub fn promote_pawn(&mut self, from: usize, to: usize, kind: PieceKind) -> bool {
        if from >= 64 || to >= 64 || from == to || matches!(kind, PieceKind::Pawn | PieceKind::King)
        {
            return false;
        }

        let Some(piece) = self.squares[from] else {
            return false;
        };
        if piece.kind != PieceKind::Pawn || !is_promotion_square(piece.color, to) {
            return false;
        }

        self.squares[from] = None;
        self.squares[to] = Some(Piece {
            color: piece.color,
            kind,
        });
        self.selected = None;
        true
    }

    /// Apply one physical-board tap.
    ///
    /// A pawn reaching its last rank is staged as a promotion and is not moved
    /// until a promotion piece is explicitly chosen.
    pub fn tap(&mut self, square: usize) -> TapResult {
        if square >= 64 {
            return TapResult::NoChange;
        }

        match self.selected {
            None => {
                if self.squares[square].is_some() {
                    self.selected = Some(square);
                    TapResult::SelectionChanged
                } else {
                    TapResult::NoChange
                }
            }
            Some(from) if from == square => {
                self.selected = None;
                TapResult::SelectionChanged
            }
            Some(from) => {
                let Some(piece) = self.squares[from] else {
                    self.selected = None;
                    return TapResult::SelectionChanged;
                };

                if piece.kind == PieceKind::Pawn && is_promotion_square(piece.color, square) {
                    self.selected = None;
                    return TapResult::Promotion {
                        from,
                        to: square,
                        color: piece.color,
                    };
                }

                if self.move_piece(from, square) {
                    TapResult::Moved { from, to: square }
                } else {
                    self.selected = None;
                    TapResult::SelectionChanged
                }
            }
        }
    }
}

const fn is_promotion_square(color: Color, square: usize) -> bool {
    match color {
        Color::White => square < 8,
        Color::Black => square >= 56,
    }
}
