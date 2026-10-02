//! Standard six-field FEN parsing used by puzzle files.

use crate::board::{Color, Piece, PieceKind};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FenError(&'static str);

impl FenError {
    pub const fn message(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for FenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for FenError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FenPosition {
    squares: [Option<Piece>; 64],
    active_color: Color,
}

impl FenPosition {
    pub const fn active_color(&self) -> Color {
        self.active_color
    }

    pub fn piece_at(&self, square: usize) -> Option<Piece> {
        self.squares.get(square).copied().flatten()
    }

    pub(crate) const fn into_squares(self) -> [Option<Piece>; 64] {
        self.squares
    }
}

pub fn parse_fen(fen: &str) -> Result<FenPosition, FenError> {
    let fields: Vec<_> = fen.split_ascii_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError("FEN must contain six fields"));
    }

    let ranks: Vec<_> = fields[0].split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError("FEN piece placement must contain eight ranks"));
    }

    let mut squares = [None; 64];
    for (rank_index, rank) in ranks.iter().enumerate() {
        let mut file = 0usize;
        for symbol in rank.chars() {
            if ('1'..='8').contains(&symbol) {
                file += symbol.to_digit(10).expect("ASCII digit") as usize;
                if file > 8 {
                    return Err(FenError("FEN rank contains more than eight squares"));
                }
                continue;
            }

            let piece = piece_from_fen_symbol(symbol)
                .ok_or(FenError("FEN contains an unknown piece symbol"))?;
            if file >= 8 {
                return Err(FenError("FEN rank contains more than eight squares"));
            }
            squares[rank_index * 8 + file] = Some(piece);
            file += 1;
        }
        if file != 8 {
            return Err(FenError(
                "each FEN rank must describe exactly eight squares",
            ));
        }
    }

    let active_color = match fields[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err(FenError("FEN active color must be w or b")),
    };

    validate_castling(fields[2])?;
    validate_en_passant(fields[3], active_color)?;

    fields[4]
        .parse::<u32>()
        .map_err(|_| FenError("FEN halfmove clock must be a nonnegative integer"))?;
    let fullmove = fields[5]
        .parse::<u32>()
        .map_err(|_| FenError("FEN fullmove number must be a positive integer"))?;
    if fullmove == 0 {
        return Err(FenError("FEN fullmove number must be a positive integer"));
    }

    Ok(FenPosition {
        squares,
        active_color,
    })
}

fn piece_from_fen_symbol(symbol: char) -> Option<Piece> {
    let (color, kind) = match symbol {
        'P' => (Color::White, PieceKind::Pawn),
        'N' => (Color::White, PieceKind::Knight),
        'B' => (Color::White, PieceKind::Bishop),
        'R' => (Color::White, PieceKind::Rook),
        'Q' => (Color::White, PieceKind::Queen),
        'K' => (Color::White, PieceKind::King),
        'p' => (Color::Black, PieceKind::Pawn),
        'n' => (Color::Black, PieceKind::Knight),
        'b' => (Color::Black, PieceKind::Bishop),
        'r' => (Color::Black, PieceKind::Rook),
        'q' => (Color::Black, PieceKind::Queen),
        'k' => (Color::Black, PieceKind::King),
        _ => return None,
    };
    Some(Piece { color, kind })
}

fn validate_castling(rights: &str) -> Result<(), FenError> {
    if rights == "-" {
        return Ok(());
    }

    let mut seen = [false; 4];
    for right in rights.bytes() {
        let index = match right {
            b'K' => 0,
            b'Q' => 1,
            b'k' => 2,
            b'q' => 3,
            _ => return Err(FenError("FEN castling rights are invalid")),
        };
        if seen[index] {
            return Err(FenError("FEN castling rights are invalid"));
        }
        seen[index] = true;
    }
    Ok(())
}

fn validate_en_passant(square: &str, active_color: Color) -> Result<(), FenError> {
    if square == "-" {
        return Ok(());
    }

    let bytes = square.as_bytes();
    let expected_rank = match active_color {
        Color::White => b'6',
        Color::Black => b'3',
    };
    if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || bytes[1] != expected_rank {
        return Err(FenError(
            "FEN en passant square is invalid for the active color",
        ));
    }
    Ok(())
}
