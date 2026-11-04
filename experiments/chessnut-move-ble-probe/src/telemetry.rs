//! M3 uses typed SDK packets; the native worker independently enforces this policy.
use crate::ipc::{Error, POSITION_UUID, RESPONSE_UUID};
use chessnut_move::protocol::{
    self, BoardEvent, Color, Command, LedColor, LedPattern, PieceKind, Position, Square, WriteKind,
};
#[derive(Clone, Debug)]
pub struct Observed {
    pub source: String,
    pub bytes: Vec<u8>,
    pub event: BoardEvent,
}
pub fn decode_observed(source: &str, bytes: &[u8]) -> Result<Observed, Error> {
    if bytes.is_empty() || bytes.len() > 512 {
        return Err(Error::MalformedNotification);
    }
    let event = match source {
        POSITION_UUID => protocol::decode_position_notification(bytes)
            .map(BoardEvent::PositionChanged)
            .map_err(|_| Error::MalformedNotification)?,
        RESPONSE_UUID => protocol::decode_response_notification(bytes)
            .map_err(|_| Error::MalformedNotification)?,
        _ => return Err(Error::MalformedNotification),
    };
    Ok(Observed {
        source: source.into(),
        bytes: bytes.into(),
        event,
    })
}
pub fn led_command(square: &str, color: &str) -> Result<Command, String> {
    let mut pattern = LedPattern::default();
    if square != "clear" {
        let square: Square = square.parse().map_err(|_| "invalid square".to_owned())?;
        let color = match color {
            "red" => LedColor::Red,
            "green" => LedColor::Green,
            "blue" => LedColor::Blue,
            "off" => LedColor::Off,
            _ => return Err("unsupported LED color".into()),
        };
        pattern.set_color(square, color);
    } else if color != "off" {
        return Err("clear requires off".into());
    }
    Ok(Command::set_leds(&pattern))
}
pub fn m3_command_allowed(command: &Command) -> bool {
    let bytes = command.bytes();
    if command.write_kind() == WriteKind::WithResponse {
        return [
            Command::enable_realtime_updates(),
            Command::read_battery_level(),
            Command::read_piece_status(),
        ]
        .iter()
        .any(|c| c.bytes() == bytes);
    }
    bytes.len() == 34
        && bytes[..2] == [0x43, 0x20]
        && bytes[2..].iter().all(|b| b & 15 <= 3 && b >> 4 <= 3)
        && bytes[2..]
            .iter()
            .map(|b| usize::from(b & 15 != 0) + usize::from(b >> 4 != 0))
            .sum::<usize>()
            <= 1
}
pub fn placement(position: &Position) -> String {
    let mut ranks = Vec::new();
    for rank in (1..=8).rev() {
        let mut row = String::new();
        let mut empty = 0;
        for file in b'a'..=b'h' {
            let square: Square = format!("{}{rank}", file as char).parse().unwrap();
            if let Some(piece) = position.piece_at(square) {
                if empty > 0 {
                    row.push(char::from(b'0' + empty));
                    empty = 0;
                }
                let c = match piece.kind {
                    PieceKind::Pawn => 'p',
                    PieceKind::Knight => 'n',
                    PieceKind::Bishop => 'b',
                    PieceKind::Rook => 'r',
                    PieceKind::Queen => 'q',
                    PieceKind::King => 'k',
                };
                row.push(if piece.color == Color::White {
                    c.to_ascii_uppercase()
                } else {
                    c
                });
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            row.push(char::from(b'0' + empty));
        }
        ranks.push(row);
    }
    ranks.join("/")
}
