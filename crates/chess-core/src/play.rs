//! Play owns transient server snapshots and input intent; it never applies local moves.
use chess_model::{parse_fen, parse_uci_move, Board, Color, PieceKind, UciMove};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const MAX_RESPONSE: usize = 256 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GameResult {
    pub outcome: String,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub api_version: u32,
    pub rating: u16,
    pub player_color: String,
    pub fen: String,
    pub moves: Vec<String>,
    pub ply: usize,
    pub status: String,
    pub result: Option<GameResult>,
    pub last_move: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub san_moves: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move_at: Option<String>,
}
fn rating_valid(rating: u16) -> bool {
    (600..=2600).contains(&rating) && rating % 100 == 0
}
pub fn color_name(color: Color) -> &'static str {
    match color {
        Color::White => "white",
        Color::Black => "black",
    }
}
impl Snapshot {
    pub fn move_label(&self, ply: usize) -> Option<&str> {
        let index = ply.checked_sub(1)?;
        self.san_moves
            .as_ref()
            .unwrap_or(&self.moves)
            .get(index)
            .map(String::as_str)
    }

    pub fn human_color(&self) -> Color {
        if self.player_color == "black" {
            Color::Black
        } else {
            Color::White
        }
    }
    pub fn parse(value: &Value) -> Result<Self, String> {
        // Null fields are part of the frozen contract, not optional omissions.
        for field in ["result", "lastMove"] {
            if value.get(field).is_none() {
                return Err(format!("Missing {field}"));
            }
        }
        let state: Self = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        let position = parse_fen(&state.fen).map_err(|e| e.to_string())?;
        if state.api_version != 1
            || !rating_valid(state.rating)
            || !matches!(state.player_color.as_str(), "white" | "black")
            || state.ply != state.moves.len()
            || state.last_move.as_ref() != state.moves.last()
            || position.fullmove_number() as usize != state.ply / 2 + 1
            || position.active_color()
                != if state.ply % 2 == 0 {
                    Color::White
                } else {
                    Color::Black
                }
            || !matches!(state.status.as_str(), "playing" | "finished")
            || (state.status == "playing"
                && (state.result.is_some() || position.active_color() != state.human_color()))
            || (state.status == "finished" && state.result.is_none())
        {
            return Err("Inconsistent server game state".into());
        }
        if let Some(result) = &state.result {
            if !matches!(result.outcome.as_str(), "white_win" | "black_win" | "draw")
                || !matches!(
                    result.reason.as_str(),
                    "checkmate" | "stalemate" | "draw" | "resignation"
                )
                || (matches!(result.reason.as_str(), "stalemate" | "draw")
                    && result.outcome != "draw")
                || (result.reason == "resignation"
                    && result.outcome
                        != if state.human_color() == Color::White {
                            "black_win"
                        } else {
                            "white_win"
                        })
            {
                return Err("Invalid game result".into());
            }
        }
        for color in [Color::White, Color::Black] {
            if (0..64)
                .filter(|&sq| {
                    position
                        .piece_at(sq)
                        .is_some_and(|p| p.color == color && p.kind == PieceKind::King)
                })
                .count()
                != 1
            {
                return Err("Server FEN must contain both kings".into());
            }
        }
        for movement in &state.moves {
            parse_uci_move(movement).map_err(|e| e.to_string())?;
        }
        if let Some(san) = &state.san_moves {
            if san.len() != state.ply
                || san.iter().any(|label| {
                    label.trim().is_empty()
                        || label.len() > 64
                        || label.chars().any(char::is_control)
                })
            {
                return Err("Invalid SAN history".into());
            }
        }
        let positions = replay_positions(&state.moves)?;
        let pieces = positions.last().expect("initial position");
        if (0..64).any(|sq| pieces.piece_at(sq) != position.piece_at(sq)) {
            return Err("History placement disagrees with FEN".into());
        }
        if state.ply == 0 && state.fen != START_FEN {
            return Err("Initial FEN mismatch".into());
        }
        if let Some(last) = &state.last_move {
            let movement = parse_uci_move(last).map_err(|e| e.to_string())?;
            let color = if state.ply % 2 == 0 {
                Color::Black
            } else {
                Color::White
            };
            if !position.piece_at(movement.to).is_some_and(|piece| {
                piece.color == color && movement.promotion.is_none_or(|kind| piece.kind == kind)
            }) || position.piece_at(movement.from).is_some()
            {
                return Err("Last move disagrees with FEN".into());
            }
        }
        Ok(state)
    }
}
/// Replay authored placements once for validation and read-only browsing; no legality engine.
pub fn replay_positions(moves: &[String]) -> Result<Vec<Board>, String> {
    // Reconstruct placement from already-authored UCI records. This is not a
    // legality checker: no move generation, check detection or rules engine.
    let initial = Board::starting_position();
    let mut pieces: [Option<chess_model::Piece>; 64] =
        std::array::from_fn(|sq| initial.piece_at(sq));
    let mut positions = vec![initial];
    for (ply, record) in moves.iter().enumerate() {
        let movement = parse_uci_move(record).map_err(|e| e.to_string())?;
        let mut piece = pieces[movement.from]
            .take()
            .ok_or("History origin is empty")?;
        if piece.color
            != if ply % 2 == 0 {
                Color::White
            } else {
                Color::Black
            }
        {
            return Err("History color mismatch".into());
        }
        if piece.kind == PieceKind::Pawn
            && movement.from % 8 != movement.to % 8
            && pieces[movement.to].is_none()
        {
            // Standard UCI en-passant records omit the captured pawn square.
            let captured = movement.from / 8 * 8 + movement.to % 8;
            pieces[captured] = None;
        }
        if piece.kind == PieceKind::King {
            let rook = match (movement.from, movement.to) {
                (60, 62) => Some((63, 61)),
                (60, 58) => Some((56, 59)),
                (4, 6) => Some((7, 5)),
                (4, 2) => Some((0, 3)),
                _ => None,
            };
            if let Some((from, to)) = rook {
                pieces[to] = pieces[from].take();
            }
        }
        if let Some(kind) = movement.promotion {
            piece.kind = kind;
        }
        pieces[movement.to] = Some(piece);
        positions.push(Board::from_squares(pieces));
    }
    Ok(positions)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Health,
    Current,
    LegacyWhiteNew { rating: u16 },
    New { rating: u16, player_color: Color },
    Move { uci: String, expected_ply: usize },
    Resign,
}
impl Request {
    pub fn wire(&self) -> (&'static str, &'static str, String) {
        match self {
            Self::Health => ("GET", "/api/v1/health", String::new()),
            Self::Current => ("GET", "/api/v1/games/current", String::new()),
            Self::LegacyWhiteNew { rating } => (
                "POST",
                "/api/v1/games",
                serde_json::json!({"rating":rating}).to_string(),
            ),
            Self::New {
                rating,
                player_color,
            } => (
                "POST",
                "/api/v1/games",
                serde_json::json!({"rating":rating,"playerColor":color_name(*player_color)})
                    .to_string(),
            ),
            Self::Move { uci, expected_ply } => (
                "POST",
                "/api/v1/games/current/moves",
                serde_json::json!({"uci":uci,"expectedPly":expected_ply}).to_string(),
            ),
            Self::Resign => ("POST", "/api/v1/games/current/resign", "{}".into()),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Response {
    Health(bool),
    Game(Snapshot),
    NoGame,
    Error {
        status: u16,
        code: String,
        message: String,
        state: Option<Snapshot>,
    },
}
pub fn parse_response(request: &Request, status: u16, bytes: &[u8]) -> Result<Response, String> {
    if bytes.len() > MAX_RESPONSE {
        return Err("Server response too large".into());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "Invalid server JSON")?;
    if (200..300).contains(&status) {
        return match (request, status) {
            (Request::Health, 200) if value["apiVersion"] == 1 => value["maiaAvailable"]
                .as_bool()
                .map(Response::Health)
                .ok_or("Invalid health response".into()),
            (Request::New { .. } | Request::LegacyWhiteNew { .. }, 201)
            | (Request::Current | Request::Move { .. } | Request::Resign, 200) => {
                let state = Snapshot::parse(&value)?;
                if matches!(request, Request::LegacyWhiteNew { .. })
                    && state.human_color() != Color::White
                {
                    return Err("Unexpected player side in legacy White response".into());
                }
                if let Request::New { player_color, .. } = request {
                    if state.human_color() != *player_color {
                        return Err("Selected side requires an updated desktop server".into());
                    }
                }
                Ok(Response::Game(state))
            }
            _ => Err("Unexpected HTTP success status".into()),
        };
    }
    let code = value["error"]["code"]
        .as_str()
        .ok_or("Missing error code")?
        .to_owned();
    let message = value["error"]["message"]
        .as_str()
        .ok_or("Missing error message")?
        .to_owned();
    if status == 404 && code == "NO_GAME" && matches!(request, Request::Current) {
        return Ok(Response::NoGame);
    }
    let state = value.get("state").map(Snapshot::parse).transpose()?;
    Ok(Response::Error {
        status,
        code,
        message,
        state,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiStatus {
    Idle,
    Connecting,
    Waiting,
    Playing,
    Error,
    Finished,
}
#[derive(Clone, Debug)]
pub struct PlayApp {
    pub player_color: Color,
    pub view_ply: usize,
    pub history_page: Option<usize>,
    positions: Vec<Board>,
    pub rating: u16,
    pub maia_available: bool,
    pub snapshot: Option<Snapshot>,
    pub board: Board,
    pub selected: Option<usize>,
    pub promotion: Option<(usize, usize)>,
    pub status: UiStatus,
    pub message: String,
    sequence: u64,
    pending: Option<(u64, Request)>,
    needs_refresh: bool,
}
impl Default for PlayApp {
    fn default() -> Self {
        Self {
            player_color: Color::White,
            view_ply: 0,
            history_page: None,
            positions: vec![Board::starting_position()],
            rating: 1500,
            maia_available: true,
            snapshot: None,
            board: Board::starting_position(),
            selected: None,
            promotion: None,
            status: UiStatus::Idle,
            message: "New Game to play White".into(),
            sequence: 0,
            pending: None,
            needs_refresh: false,
        }
    }
}
impl PlayApp {
    pub fn viewing_history(&self) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| self.view_ply < s.ply)
    }
    pub fn view_history(&mut self, ply: usize) -> bool {
        if self.in_flight() {
            return false;
        }
        let Some(snapshot) = &self.snapshot else {
            return false;
        };
        if ply > snapshot.ply {
            return false;
        }
        self.board = if ply == snapshot.ply {
            Board::from_fen(&snapshot.fen).expect("validated FEN")
        } else {
            self.positions[ply].clone()
        };
        self.view_ply = ply;
        self.cancel_selection();
        true
    }
    pub fn latest(&mut self) {
        if let Some(ply) = self.snapshot.as_ref().map(|s| s.ply) {
            if self.view_history(ply) {
                self.history_page = None;
            }
        }
    }
    pub fn history_page(&mut self, page: usize) {
        if !self.in_flight() {
            self.history_page = Some(page);
        }
    }
    pub fn worker_failed(&mut self) {
        if let Some((id, _)) = &self.pending {
            self.complete(*id, Err("HTTP worker stopped".into()));
        }
    }
    pub fn in_flight(&self) -> bool {
        self.pending.is_some()
    }
    pub fn can_move(&self) -> bool {
        !self.in_flight()
            && !self.viewing_history()
            && !self.needs_refresh
            && self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.status == "playing")
    }
    pub fn can_new(&self) -> bool {
        !self.in_flight()
            && !self.viewing_history()
            && !self.needs_refresh
            && self
                .snapshot
                .as_ref()
                .is_none_or(|s| s.status == "finished")
    }
    pub fn flipped(&self) -> bool {
        self.snapshot
            .as_ref()
            .map_or(self.player_color, Snapshot::human_color)
            == Color::Black
    }
    pub fn choose_color(&mut self, color: Color) {
        if self.can_new() {
            self.player_color = color;
            if self.snapshot.is_none() {
                self.message = format!(
                    "New Game to play {}",
                    if color == Color::White {
                        "White"
                    } else {
                        "Black"
                    }
                );
            }
            self.cancel_selection();
        }
    }
    pub fn new_game_request(&self) -> Request {
        Request::New {
            rating: self.rating,
            player_color: self.player_color,
        }
    }
    pub fn change_rating(&mut self, delta: i16) {
        if self.can_new() && matches!(delta, -100 | 100) {
            self.rating = (self.rating as i16 + delta).clamp(600, 2600) as u16;
        }
    }
    pub fn cancel_selection(&mut self) {
        self.selected = None;
        self.promotion = None;
    }
    pub fn tap(&mut self, square: usize) -> Option<Request> {
        if !self.can_move() || square >= 64 || self.promotion.is_some() {
            return None;
        }
        if self.selected == Some(square) {
            self.selected = None;
            return None;
        }
        if self
            .board
            .piece_at(square)
            .is_some_and(|p| p.color == self.player_color)
        {
            self.selected = Some(square);
            return None;
        }
        let from = self.selected?;
        if self.board.piece_at(from)?.kind == PieceKind::Pawn
            && if self.player_color == Color::White {
                square < 8
            } else {
                square >= 56
            }
        {
            self.promotion = Some((from, square));
            return None;
        }
        self.move_request(from, square, None)
    }
    fn move_request(
        &mut self,
        from: usize,
        to: usize,
        promotion: Option<PieceKind>,
    ) -> Option<Request> {
        let uci = UciMove::new(from, to, promotion).ok()?.to_string();
        self.cancel_selection();
        Some(Request::Move {
            uci,
            expected_ply: self.snapshot.as_ref()?.ply,
        })
    }
    pub fn promote(&mut self, kind: PieceKind) -> Option<Request> {
        if !self.can_move() {
            return None;
        }
        let (from, to) = self.promotion?;
        self.move_request(from, to, Some(kind))
    }
    pub fn begin(&mut self, request: Request) -> Option<u64> {
        if self.in_flight() {
            return None;
        }
        match &request {
            Request::New {
                rating,
                player_color,
            } if !self.can_new()
                || !rating_valid(*rating)
                || *player_color != self.player_color =>
            {
                return None
            }
            Request::LegacyWhiteNew { rating }
                if !self.can_new()
                    || !rating_valid(*rating)
                    || self.player_color != Color::White =>
            {
                return None
            }
            Request::Move { uci, expected_ply }
                if !self.can_move()
                    || parse_uci_move(uci).is_err()
                    || self.snapshot.as_ref()?.ply != *expected_ply =>
            {
                return None
            }
            Request::Resign if !self.can_move() => return None,
            _ => {}
        }
        self.cancel_selection();
        self.sequence = self.sequence.wrapping_add(1);
        self.status = if matches!(request, Request::Health | Request::Current) {
            UiStatus::Connecting
        } else {
            UiStatus::Waiting
        };
        self.message = if matches!(request, Request::Move { .. }) {
            "Maia thinking…"
        } else {
            "Connecting…"
        }
        .into();
        self.pending = Some((self.sequence, request));
        Some(self.sequence)
    }
    fn apply(&mut self, snapshot: Snapshot) {
        self.board = Board::from_fen(&snapshot.fen).expect("validated snapshot");
        self.positions = replay_positions(&snapshot.moves).expect("validated history");
        self.view_ply = snapshot.ply;
        self.history_page = None;
        self.rating = snapshot.rating;
        self.player_color = snapshot.human_color();
        self.status = if snapshot.status == "finished" {
            UiStatus::Finished
        } else {
            UiStatus::Playing
        };
        self.message = match &snapshot.result {
            Some(r) => format!("{}: {}", r.outcome.replace('_', " "), r.reason),
            None => if self.player_color == Color::White {
                "White to move"
            } else {
                "Black to move"
            }
            .into(),
        };
        if !self.maia_available && snapshot.status == "playing" {
            self.message = "Maia unavailable. Refresh server state".into();
        }
        self.snapshot = Some(snapshot);
        self.needs_refresh = false;
        self.cancel_selection();
    }
    pub fn complete(&mut self, id: u64, response: Result<Response, String>) -> Option<Request> {
        if self.pending.as_ref().map(|p| p.0) != Some(id) {
            return None;
        }
        let request = self.pending.take().expect("matching pending request").1;
        match response {
            Ok(Response::Health(available)) => {
                self.maia_available = available;
                return Some(Request::Current);
            }
            Ok(Response::Game(state)) => self.apply(state),
            Ok(Response::NoGame) => {
                self.snapshot = None;
                self.view_ply = 0;
                self.history_page = None;
                self.positions = vec![Board::starting_position()];
                self.board = Board::starting_position();
                self.status = UiStatus::Idle;
                self.needs_refresh = false;
                self.message = if self.maia_available {
                    if self.player_color == Color::White {
                        "No game. New Game to play White"
                    } else {
                        "No game. New Game to play Black"
                    }
                } else {
                    "Maia unavailable. Refresh server state"
                }
                .into();
                if !self.maia_available {
                    self.status = UiStatus::Error;
                    self.needs_refresh = true;
                }
            }
            Ok(Response::Error {
                status,
                code,
                message,
                state,
            }) => {
                if status == 400 && code == "BAD_REQUEST" {
                    if let Request::New {
                        rating,
                        player_color: Color::White,
                    } = request
                    {
                        // A definite validation failure cannot have committed a game.
                        // Retry once in legacy White format; never do this after transport failure.
                        return Some(Request::LegacyWhiteNew { rating });
                    }
                }
                if status == 409 {
                    if let Some(state) = state {
                        self.apply(state);
                        if self.status == UiStatus::Finished {
                            return None;
                        }
                    } else {
                        self.needs_refresh = true;
                    }
                }
                self.status = UiStatus::Error;
                self.message = if matches!(
                    request,
                    Request::New {
                        player_color: Color::Black,
                        ..
                    }
                ) && status == 400
                {
                    self.needs_refresh = true;
                    "Black play requires an updated desktop server. Refresh server state".into()
                } else if status == 422 {
                    "Illegal move".into()
                } else if status == 409 && !self.needs_refresh {
                    "Server state updated; select your move again".into()
                } else {
                    self.needs_refresh = true;
                    format!("{code}: {message}. Refresh server state")
                };
            }
            Err(error) => {
                self.status = UiStatus::Error;
                self.needs_refresh = true;
                self.message = format!("{error}. Refresh server state");
            }
        }
        None
    }
}
