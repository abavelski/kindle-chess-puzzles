pub mod discovery;
pub mod ipc;
pub mod m3_session;
pub mod preflight;
pub mod session;
pub mod telemetry;

use chessnut_move::protocol::{AutoMoveMode, Color, Command, Piece, PieceKind, Position};

/// Full host-side FEN plus the placement-only SDK representation.
/// This validates syntax, not legal chess or physical motor feasibility.
#[derive(Debug)]
pub struct TargetFen {
    pub fen: String,
    pub position: Position,
}
impl TargetFen {
    pub fn parse(fen: &str) -> Result<Self, &'static str> {
        if fen.len() > 256 || !fen.is_ascii() {
            return Err("FEN must be ASCII and at most 256 bytes");
        }
        let fields: Vec<_> = fen.split_whitespace().collect();
        if fields.len() != 6 {
            return Err("FEN requires six fields");
        }
        if !matches!(fields[1], "w" | "b") {
            return Err("invalid active color");
        }
        if fields[2] != "-" {
            let mut last = None;
            for c in fields[2].chars() {
                let index = "KQkq".find(c).ok_or("invalid castling rights")?;
                if last.is_some_and(|last| index <= last) {
                    return Err("duplicate or unordered castling rights");
                }
                last = Some(index);
            }
        }
        let ep = fields[3].as_bytes();
        if fields[3] != "-"
            && !(ep.len() == 2 && (b'a'..=b'h').contains(&ep[0]) && matches!(ep[1], b'3' | b'6'))
        {
            return Err("invalid en passant square");
        }
        for (index, field) in fields[4..].iter().enumerate() {
            if !field.bytes().all(|c| c.is_ascii_digit())
                || field.parse::<u32>().is_err()
                || (index == 1 && field.parse::<u32>() == Ok(0))
            {
                return Err("invalid move counter");
            }
        }
        let ranks: Vec<_> = fields[0].split('/').collect();
        if ranks.len() != 8 {
            return Err("placement requires eight ranks");
        }
        let mut squares = [None; 64];
        for (rank, text) in ranks.iter().enumerate() {
            let mut file = 0;
            let mut previous_digit = false;
            for c in text.chars() {
                if let Some(n) = c.to_digit(10) {
                    if !(1..=8).contains(&n) || previous_digit {
                        return Err("invalid empty-square run");
                    }
                    file += n as usize;
                    previous_digit = true;
                } else {
                    let kind = match c.to_ascii_lowercase() {
                        'p' => PieceKind::Pawn,
                        'r' => PieceKind::Rook,
                        'n' => PieceKind::Knight,
                        'b' => PieceKind::Bishop,
                        'q' => PieceKind::Queen,
                        'k' => PieceKind::King,
                        _ => return Err("invalid piece character"),
                    };
                    if file >= 8 {
                        return Err("rank exceeds eight files");
                    }
                    squares[(7 - rank) * 8 + file] = Some(Piece {
                        kind,
                        color: if c.is_ascii_uppercase() {
                            Color::White
                        } else {
                            Color::Black
                        },
                    });
                    file += 1;
                    previous_digit = false;
                }
                if file > 8 {
                    return Err("rank exceeds eight files");
                }
            }
            if file != 8 {
                return Err("rank requires eight files");
            }
        }
        Ok(Self {
            fen: fen.to_owned(),
            position: Position::new(squares),
        })
    }
    /// Encoding only: this type has no transport and cannot move a board.
    pub fn packet(&self) -> Command {
        Command::auto_move(self.position, AutoMoveMode::Normal)
    }
}
