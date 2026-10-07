//! Deterministic state for the game-review workspace.

use crate::{
    app::PendingPromotion, AnalysisNodeIndex, Board, Color, PieceKind, ReviewGame, ReviewGameKey,
    TapResult,
};

pub const REVIEW_GAMES_PER_PAGE: usize = 6;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Workspace {
    #[default]
    Puzzles,
    Review,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewGameEntry {
    key: ReviewGameKey,
    label: String,
    game: ReviewGame,
}

impl ReviewGameEntry {
    pub fn new(collection_id: impl Into<String>, game: ReviewGame) -> Self {
        let key = game.key(collection_id);
        let metadata = &game.metadata;
        let label = format!(
            "{} - {}\n{} / {}  {}",
            metadata.white, metadata.black, metadata.event, metadata.date, metadata.result
        );
        Self { key, label, game }
    }

    pub const fn key(&self) -> &ReviewGameKey {
        &self.key
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn game(&self) -> &ReviewGame {
        &self.game
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewFileError {
    collection_id: String,
    error: String,
}

impl ReviewFileError {
    pub fn new(collection_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            collection_id: collection_id.into(),
            error: error.into(),
        }
    }

    pub fn collection_id(&self) -> &str {
        &self.collection_id
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewState {
    active_game_index: usize,
    selected_node: AnalysisNodeIndex,
    main_line_ply: usize,
    analysis_page: usize,
    analysis_focus: Option<AnalysisNodeIndex>,
    flipped: bool,
    orientation_locked: bool,
    game_picker_open: bool,
    game_picker_page: usize,
    free_board: bool,
    authored_board: Board,
    scratch_board: Option<Board>,
    pending_promotion: Option<PendingPromotion>,
}

impl ReviewState {
    pub(crate) fn new(active_game_index: usize, game: &ReviewGame) -> Self {
        let selected_node = game.analysis.root_index();
        let authored_board =
            Board::from_fen(&game.fen).expect("validated review game FEN remains valid");
        Self {
            active_game_index,
            selected_node,
            main_line_ply: 0,
            analysis_page: 0,
            analysis_focus: Some(selected_node),
            flipped: game.analysis.root().position.active_color() == Color::Black,
            orientation_locked: false,
            game_picker_open: false,
            game_picker_page: 0,
            free_board: false,
            authored_board,
            scratch_board: None,
            pending_promotion: None,
        }
    }

    pub const fn active_game_index(&self) -> usize {
        self.active_game_index
    }

    pub const fn selected_node(&self) -> AnalysisNodeIndex {
        self.selected_node
    }

    pub const fn main_line_ply(&self) -> usize {
        self.main_line_ply
    }

    pub const fn analysis_page(&self) -> usize {
        self.analysis_page
    }

    pub const fn analysis_focus(&self) -> Option<AnalysisNodeIndex> {
        self.analysis_focus
    }

    pub const fn flipped(&self) -> bool {
        self.flipped
    }

    pub const fn orientation_locked(&self) -> bool {
        self.orientation_locked
    }

    pub const fn game_picker_open(&self) -> bool {
        self.game_picker_open
    }

    pub const fn game_picker_page(&self) -> usize {
        self.game_picker_page
    }

    pub const fn free_board_enabled(&self) -> bool {
        self.free_board
    }

    pub fn board(&self) -> &Board {
        self.scratch_board.as_ref().unwrap_or(&self.authored_board)
    }

    pub const fn authored_board(&self) -> &Board {
        &self.authored_board
    }

    pub const fn pending_promotion(&self) -> Option<&PendingPromotion> {
        self.pending_promotion.as_ref()
    }

    pub fn can_previous(&self) -> bool {
        self.main_line_ply > 0
    }

    pub fn can_next(&self, game: &ReviewGame) -> bool {
        game.analysis
            .main_line_node_at_ply(self.main_line_ply.saturating_add(1))
            .is_some()
    }

    pub fn game_picker_page_count(&self, total_games: usize) -> usize {
        total_games.div_ceil(REVIEW_GAMES_PER_PAGE)
    }

    pub fn game_picker_visible_range(&self, total_games: usize) -> std::ops::Range<usize> {
        let start = self
            .game_picker_page
            .saturating_mul(REVIEW_GAMES_PER_PAGE)
            .min(total_games);
        let end = start.saturating_add(REVIEW_GAMES_PER_PAGE).min(total_games);
        start..end
    }

    pub const fn game_picker_can_previous_page(&self) -> bool {
        self.game_picker_page > 0
    }

    pub fn game_picker_can_next_page(&self, total_games: usize) -> bool {
        self.game_picker_page + 1 < self.game_picker_page_count(total_games)
    }

    pub(crate) fn select_node(&mut self, game: &ReviewGame, index: AnalysisNodeIndex) {
        if index == self.selected_node {
            return;
        }
        let Some(node) = game.analysis.node(index) else {
            return;
        };
        let Some(anchor) = game.analysis.nearest_main_line_ancestor(index) else {
            return;
        };
        let Some(main_line_ply) = game.analysis.main_line_ply(anchor) else {
            return;
        };

        self.authored_board =
            Board::from_fen(&node.fen).expect("validated review node FEN remains valid");
        self.selected_node = index;
        self.main_line_ply = main_line_ply;
        self.analysis_focus = Some(index);
        self.pending_promotion = None;
        if self.free_board {
            self.scratch_board = Some(self.authored_board.clone());
        }
    }

    pub(crate) fn navigate_main_line(&mut self, game: &ReviewGame, forward: bool) {
        let target_ply = if forward {
            self.main_line_ply
                .checked_add(1)
                .filter(|ply| game.analysis.main_line_node_at_ply(*ply).is_some())
        } else {
            self.main_line_ply.checked_sub(1)
        };
        let Some(target_ply) = target_ply else {
            return;
        };
        let Some(target) = game.analysis.main_line_node_at_ply(target_ply) else {
            return;
        };
        self.select_node(game, target);
    }

    pub(crate) fn analysis_previous_page(&mut self) {
        self.analysis_page = self.analysis_page.saturating_sub(1);
        self.analysis_focus = None;
    }

    pub(crate) fn analysis_next_page(&mut self) {
        self.analysis_page = self.analysis_page.saturating_add(1);
        self.analysis_focus = None;
    }

    pub(crate) fn flip(&mut self) {
        self.flipped = !self.flipped;
    }

    pub(crate) fn toggle_orientation_lock(&mut self) {
        self.orientation_locked = !self.orientation_locked;
    }

    pub(crate) fn toggle_free(&mut self) {
        self.set_free_enabled(!self.free_board);
    }

    pub(crate) fn set_free_enabled(&mut self, enabled: bool) {
        if enabled == self.free_board {
            return;
        }
        self.pending_promotion = None;
        self.free_board = enabled;
        self.scratch_board = enabled.then(|| self.authored_board.clone());
    }

    pub(crate) fn reset_scratch(&mut self) {
        if self.free_board {
            self.pending_promotion = None;
            self.scratch_board = Some(self.authored_board.clone());
        }
    }

    pub(crate) fn handle_square_tap(&mut self, square: usize) {
        if !self.free_board || self.pending_promotion.is_some() {
            return;
        }
        let Some(before) = self.scratch_board.clone() else {
            return;
        };
        let Some(board) = self.scratch_board.as_mut() else {
            return;
        };
        if let TapResult::Promotion { from, to, color } = board.tap(square) {
            self.pending_promotion = Some(PendingPromotion {
                before,
                from,
                to,
                color,
            });
        }
    }

    pub(crate) fn finish_promotion(&mut self, kind: PieceKind) {
        let Some(pending) = self.pending_promotion.take() else {
            return;
        };
        let Some(board) = self.scratch_board.as_mut() else {
            return;
        };
        let before = pending.before;
        *board = before.clone();
        board.clear_selection();
        if !board.promote_pawn(pending.from, pending.to, kind) {
            *board = before;
            board.clear_selection();
        }
    }

    pub(crate) fn cancel_promotion(&mut self) {
        let Some(pending) = self.pending_promotion.take() else {
            return;
        };
        if let Some(board) = self.scratch_board.as_mut() {
            *board = pending.before;
            board.clear_selection();
        }
    }

    pub(crate) fn open_game_picker(&mut self, total_games: usize) {
        if total_games == 0 || self.game_picker_open {
            return;
        }
        self.game_picker_open = true;
        self.game_picker_page = self.active_game_index / REVIEW_GAMES_PER_PAGE;
    }

    pub(crate) fn close_game_picker(&mut self) {
        self.game_picker_open = false;
    }

    pub(crate) fn game_picker_previous_page(&mut self) {
        self.game_picker_page = self.game_picker_page.saturating_sub(1);
    }

    pub(crate) fn game_picker_next_page(&mut self, total_games: usize) {
        if self.game_picker_can_next_page(total_games) {
            self.game_picker_page += 1;
        }
    }

    pub(crate) fn activate_game(&mut self, index: usize, game: &ReviewGame) {
        let flipped = self.flipped;
        let orientation_locked = self.orientation_locked;
        *self = Self::new(index, game);
        if orientation_locked {
            self.orientation_locked = true;
            self.flipped = flipped;
        }
    }
}
