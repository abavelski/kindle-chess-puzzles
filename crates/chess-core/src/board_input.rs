//! Physical placement differences, using only the existing board move model.
use crate::{Board, PieceKind, UciMove};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BoardInputStatus {
    #[default]
    Off,
    WaitingForPosition,
    Listening,
    MatchKindle,
}

pub fn infer_board_move(before: &Board, after: &Board) -> Option<UciMove> {
    let changed: Vec<_> = (0..64)
        .filter(|&sq| before.piece_at(sq) != after.piece_at(sq))
        .collect();
    if changed.len() != 2 {
        return None;
    }
    let from = *changed
        .iter()
        .find(|&&sq| before.piece_at(sq).is_some() && after.piece_at(sq).is_none())?;
    let to = *changed
        .iter()
        .find(|&&sq| sq != from && after.piece_at(sq).is_some())?;
    let moving = before.piece_at(from)?;
    let arrived = after.piece_at(to)?;
    if moving.color != arrived.color {
        return None;
    }
    // Do not mistake an unfinished compound move for an ordinary move. The
    // runtime deliberately has no castling or en-passant implementation.
    if moving.kind == PieceKind::King && from / 8 == to / 8 && from.abs_diff(to) == 2 {
        return None;
    }
    if moving.kind == PieceKind::Pawn && from % 8 != to % 8 && before.piece_at(to).is_none() {
        return None;
    }
    if moving.kind == PieceKind::Rook {
        for (rook_from, rook_to, king) in [(63, 61, 60), (56, 59, 60), (7, 5, 4), (0, 3, 4)] {
            if from == rook_from
                && to == rook_to
                && before.piece_at(king).is_some_and(|piece| {
                    piece.kind == PieceKind::King && piece.color == moving.color
                })
            {
                return None;
            }
        }
    }
    let last_rank = match moving.color {
        crate::Color::White => to < 8,
        crate::Color::Black => to >= 56,
    };
    let promotion = if moving.kind == PieceKind::Pawn && last_rank {
        if matches!(arrived.kind, PieceKind::Pawn | PieceKind::King) {
            return None;
        }
        Some(arrived.kind)
    } else {
        if moving != arrived {
            return None;
        }
        None
    };
    let movement = UciMove::new(from, to, promotion).ok()?;
    let mut expected = before.clone();
    (expected.apply_uci_move(movement) && expected.placement() == after.placement())
        .then_some(movement)
}
