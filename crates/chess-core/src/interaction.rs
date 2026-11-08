//! Shared permissive board interaction values, independent of either product.
use crate::{Board, Color, PieceKind};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionChoice {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PromotionChoice {
    pub const fn piece_kind(self) -> PieceKind {
        match self {
            Self::Queen => PieceKind::Queen,
            Self::Rook => PieceKind::Rook,
            Self::Bishop => PieceKind::Bishop,
            Self::Knight => PieceKind::Knight,
        }
    }
}

impl TryFrom<usize> for PromotionChoice {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Queen),
            1 => Ok(Self::Rook),
            2 => Ok(Self::Bishop),
            3 => Ok(Self::Knight),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPromotion {
    pub(crate) before: Board,
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) color: Color,
}

impl PendingPromotion {
    pub const fn from(&self) -> usize {
        self.from
    }

    pub const fn to(&self) -> usize {
        self.to
    }

    pub const fn color(&self) -> Color {
        self.color
    }
}
