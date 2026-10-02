//! Square/algebraic conversion and the UCI move subset used by puzzle files.

use crate::board::PieceKind;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UciMove {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<PieceKind>,
}

impl UciMove {
    pub fn new(from: usize, to: usize, promotion: Option<PieceKind>) -> Result<Self, UciError> {
        if from >= 64 || to >= 64 {
            return Err(UciError("UCI square is outside the board"));
        }
        if from == to {
            return Err(UciError("UCI origin and destination must differ"));
        }
        if promotion.is_some_and(|kind| matches!(kind, PieceKind::Pawn | PieceKind::King)) {
            return Err(UciError("UCI promotion must be q, r, b, or n"));
        }
        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = square_to_algebraic(self.from).ok_or(fmt::Error)?;
        let to = square_to_algebraic(self.to).ok_or(fmt::Error)?;
        formatter.write_str(&from)?;
        formatter.write_str(&to)?;
        if let Some(kind) = self.promotion {
            formatter.write_str(match kind {
                PieceKind::Queen => "q",
                PieceKind::Rook => "r",
                PieceKind::Bishop => "b",
                PieceKind::Knight => "n",
                PieceKind::Pawn | PieceKind::King => return Err(fmt::Error),
            })?;
        }
        Ok(())
    }
}

impl FromStr for UciMove {
    type Err = UciError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_uci_move(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UciError(&'static str);

impl UciError {
    pub const fn message(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for UciError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for UciError {}

pub fn square_to_algebraic(square: usize) -> Option<String> {
    if square >= 64 {
        return None;
    }
    let file = (b'a' + (square % 8) as u8) as char;
    let rank = (b'8' - (square / 8) as u8) as char;
    Some(format!("{file}{rank}"))
}

pub fn algebraic_to_square(square: &str) -> Option<usize> {
    square_from_bytes(square.as_bytes())
}

fn square_from_bytes(bytes: &[u8]) -> Option<usize> {
    if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1])
    {
        return None;
    }
    let file = usize::from(bytes[0] - b'a');
    let rank_from_top = usize::from(b'8' - bytes[1]);
    Some(rank_from_top * 8 + file)
}

pub fn parse_uci_move(value: &str) -> Result<UciMove, UciError> {
    let bytes = value.as_bytes();
    if bytes.len() != 4 && bytes.len() != 5 {
        return Err(UciError(
            "UCI move must contain four characters plus an optional promotion suffix",
        ));
    }

    let from = square_from_bytes(&bytes[..2]).ok_or(UciError("UCI origin square is invalid"))?;
    let to =
        square_from_bytes(&bytes[2..4]).ok_or(UciError("UCI destination square is invalid"))?;
    let promotion = if bytes.len() == 5 {
        Some(match bytes[4] {
            b'q' => PieceKind::Queen,
            b'r' => PieceKind::Rook,
            b'b' => PieceKind::Bishop,
            b'n' => PieceKind::Knight,
            _ => return Err(UciError("UCI promotion must be q, r, b, or n")),
        })
    } else {
        None
    };

    UciMove::new(from, to, promotion)
}
