//! Conservative physical probe geometry; this is not a chess legality engine.
use chessnut_move::protocol::{AutoMoveMode, Color, Command, PieceKind, Position, Square};
use std::time::{Duration, Instant};
fn square(i: usize) -> Square {
    format!("{}{}", (b'a' + (i % 8) as u8) as char, i / 8 + 1)
        .parse()
        .unwrap()
}
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub source: Position,
    pub target: Position,
}
impl Plan {
    pub fn new(source: Position, target: Position) -> Result<Self, &'static str> {
        let mut removed = Vec::new();
        let mut added = Vec::new();
        let mut counts = [[0usize; 6]; 2];
        for i in 0..64 {
            let a = source.piece_at(square(i));
            let b = target.piece_at(square(i));
            if let Some(p) = a {
                let color = usize::from(p.color == Color::Black);
                let kind = match p.kind {
                    PieceKind::Pawn => 0,
                    PieceKind::Rook => 1,
                    PieceKind::Knight => 2,
                    PieceKind::Bishop => 3,
                    PieceKind::Queen => 4,
                    PieceKind::King => 5,
                };
                counts[color][kind] += 1;
            }
            if a != b {
                if let Some(p) = a {
                    removed.push((i, p));
                }
                if let Some(p) = b {
                    added.push((i, p));
                }
            }
        }
        if counts.iter().any(|c| {
            c[5] != 1 || c[0] > 8 || c.iter().sum::<usize>() > 16 || c[1..5].iter().any(|v| *v > 2)
        }) {
            return Err("source must have one king per color and supported piece counts");
        }
        if removed.len() != 1 || added.len() != 1 {
            return Err("only one pawn relocation without capture is supported");
        }
        let (from, piece) = removed[0];
        let (to, other) = added[0];
        if piece != other
            || piece.kind != PieceKind::Pawn
            || from % 8 != to % 8
            || source.piece_at(square(to)).is_some()
            || target.piece_at(square(from)).is_some()
        {
            return Err("only a same-file pawn relocation into an empty square is supported");
        }
        let low = from.min(to);
        let high = from.max(to);
        // Explicit user-requested longer STOP probe after short moves passed.
        let stop_probe = low == 12 && high == 52 && piece.color == Color::White;
        if (high - low > 16 && !stop_probe) || low / 8 == 0 || high / 8 == 7 {
            return Err("only one/two ranks or the explicit e2-e7 STOP probe are supported");
        }
        for i in ((low + 8)..high).step_by(8) {
            if source.piece_at(square(i)).is_some() {
                return Err("pawn travel path is occupied");
            }
        }
        Ok(Self { source, target })
    }
    pub fn command(&self) -> Command {
        Command::auto_move(self.target, AutoMoveMode::Normal)
    }
}
#[derive(Default)]
pub struct Guard {
    enabled: bool,
    pending: Option<(Plan, Instant)>,
}
impl Guard {
    pub fn enable(&mut self) {
        self.enabled = true;
        self.pending = None;
    }
    pub fn reset(&mut self) {
        self.enabled = false;
        self.pending = None;
    }
    pub fn preview(&mut self, plan: Plan, now: Instant) -> Result<(), &'static str> {
        if !self.enabled {
            return Err("use motion-enable before preparing each target");
        }
        self.pending = Some((plan, now));
        Ok(())
    }
    pub fn confirm(&mut self, current: Position, now: Instant) -> Result<Plan, &'static str> {
        let pending = self.pending.take();
        let enabled = self.enabled;
        self.enabled = false;
        let (plan, created) = pending.ok_or("no pending target; nothing sent")?;
        if !enabled || now.saturating_duration_since(created) > Duration::from_secs(30) {
            return Err("confirmation expired; nothing sent");
        }
        if current != plan.source {
            return Err("sensor position changed since preview; nothing sent");
        }
        Ok(plan)
    }
}
