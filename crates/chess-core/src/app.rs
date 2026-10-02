//! Pure application state for solving and exploring puzzle collections.

use crate::{
    parse_uci_move, Board, CollectionEntry, Color, PieceKind, Progress, Puzzle, PuzzleCollection,
    TapResult, UciMove,
};

pub const COLLECTIONS_PER_PAGE: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveCollection {
    key: String,
    title: Option<String>,
    puzzles: Vec<Puzzle>,
}

impl ActiveCollection {
    pub fn from_collection(key: impl Into<String>, collection: PuzzleCollection) -> Self {
        Self {
            key: key.into(),
            title: collection.title,
            puzzles: collection.puzzles,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn puzzles(&self) -> &[Puzzle] {
        &self.puzzles
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BoardMode {
    #[default]
    Solution,
    FreeBoard,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SolutionFeedback {
    #[default]
    None,
    Correct,
    Wrong,
    Complete,
}

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
    before: Board,
    from: usize,
    to: usize,
    color: Color,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    TapSquare(usize),
    ChoosePromotion(PromotionChoice),
    CancelPromotion,
    PreviousPuzzle,
    NextPuzzle,
    Reset,
    Flip,
    ToggleMode,
    ToggleOrientationLock,
    ToggleDescription,
    OpenCollectionPicker,
    CloseCollectionPicker,
    CollectionPickerPreviousPage,
    CollectionPickerNextPage,
    SelectCollection(usize),
    ActivateCollection(ActiveCollection),
    SetTransientMessage(Option<String>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    ProgressChanged,
    CollectionRequested(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    active_collection: ActiveCollection,
    puzzle_index: usize,
    board: Board,
    mode: BoardMode,
    solution_ply: usize,
    feedback: SolutionFeedback,
    pending_promotion: Option<PendingPromotion>,
    flipped: bool,
    orientation_locked: bool,
    description_visible: bool,
    progress: Progress,
    transient_message: Option<String>,
    collection_entries: Vec<CollectionEntry>,
    collection_picker_open: bool,
    collection_picker_page: usize,
}

impl AppState {
    pub fn new(active_collection: ActiveCollection, progress: Progress) -> Self {
        let puzzle_index = remembered_puzzle_index(&active_collection, &progress);
        let puzzle = active_collection
            .puzzles
            .get(puzzle_index)
            .expect("parsed puzzle collections are non-empty");
        let board = Board::from_fen(&puzzle.fen).expect("parsed puzzle FEN remains valid");
        let flipped = puzzle.side_to_move() == Color::Black;
        let key = active_collection.key.clone();
        let puzzle_id = puzzle.id.clone();
        let mut progress = progress;
        progress.set_active_file(&key);
        progress.remember_puzzle(&key, &puzzle_id);

        Self {
            active_collection,
            puzzle_index,
            board,
            mode: BoardMode::Solution,
            solution_ply: 0,
            feedback: SolutionFeedback::None,
            pending_promotion: None,
            flipped,
            orientation_locked: false,
            description_visible: false,
            progress,
            transient_message: None,
            collection_entries: Vec::new(),
            collection_picker_open: false,
            collection_picker_page: 0,
        }
    }

    pub fn active_collection(&self) -> &ActiveCollection {
        &self.active_collection
    }

    pub const fn active_puzzle_index(&self) -> usize {
        self.puzzle_index
    }

    pub const fn can_previous_puzzle(&self) -> bool {
        self.puzzle_index > 0
    }

    pub fn can_next_puzzle(&self) -> bool {
        self.puzzle_index + 1 < self.active_collection.puzzles.len()
    }

    pub fn active_puzzle(&self) -> &Puzzle {
        &self.active_collection.puzzles[self.puzzle_index]
    }

    pub const fn board(&self) -> &Board {
        &self.board
    }

    pub const fn mode(&self) -> BoardMode {
        self.mode
    }

    pub const fn solution_ply(&self) -> usize {
        self.solution_ply
    }

    pub const fn feedback(&self) -> SolutionFeedback {
        self.feedback
    }

    pub fn pending_promotion(&self) -> Option<&PendingPromotion> {
        self.pending_promotion.as_ref()
    }

    pub const fn flipped(&self) -> bool {
        self.flipped
    }

    pub const fn orientation_locked(&self) -> bool {
        self.orientation_locked
    }

    pub const fn description_visible(&self) -> bool {
        self.description_visible
    }

    pub const fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn transient_message(&self) -> Option<&str> {
        self.transient_message.as_deref()
    }

    pub fn is_current_solved(&self) -> bool {
        self.progress
            .is_solved(self.active_collection.key(), &self.active_puzzle().id)
    }

    pub fn set_collection_entries(&mut self, entries: Vec<CollectionEntry>) {
        self.collection_entries = entries;
        self.collection_picker_open = false;
        self.collection_picker_page = 0;
    }

    pub fn collection_entries(&self) -> &[CollectionEntry] {
        &self.collection_entries
    }

    pub const fn collection_picker_open(&self) -> bool {
        self.collection_picker_open
    }

    pub const fn collection_picker_page(&self) -> usize {
        self.collection_picker_page
    }

    pub fn collection_picker_page_count(&self) -> usize {
        self.collection_entries.len().div_ceil(COLLECTIONS_PER_PAGE)
    }

    pub fn collection_picker_visible_range(&self) -> std::ops::Range<usize> {
        let start = self
            .collection_picker_page
            .saturating_mul(COLLECTIONS_PER_PAGE)
            .min(self.collection_entries.len());
        let end = start
            .saturating_add(COLLECTIONS_PER_PAGE)
            .min(self.collection_entries.len());
        start..end
    }

    pub const fn collection_picker_can_previous_page(&self) -> bool {
        self.collection_picker_page > 0
    }

    pub fn collection_picker_can_next_page(&self) -> bool {
        self.collection_picker_page + 1 < self.collection_picker_page_count()
    }

    pub fn dispatch(&mut self, action: Action) -> Vec<Effect> {
        if self.collection_picker_open
            && !matches!(
                &action,
                Action::CloseCollectionPicker
                    | Action::CollectionPickerPreviousPage
                    | Action::CollectionPickerNextPage
                    | Action::SelectCollection(_)
                    | Action::ActivateCollection(_)
                    | Action::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        match action {
            Action::OpenCollectionPicker => {
                self.open_collection_picker();
                Vec::new()
            }
            Action::CloseCollectionPicker => {
                self.collection_picker_open = false;
                Vec::new()
            }
            Action::CollectionPickerPreviousPage => {
                self.collection_picker_page = self.collection_picker_page.saturating_sub(1);
                Vec::new()
            }
            Action::CollectionPickerNextPage => {
                if self.collection_picker_can_next_page() {
                    self.collection_picker_page += 1;
                }
                Vec::new()
            }
            Action::SelectCollection(index) => self.request_collection(index),
            Action::TapSquare(square) => Self::progress_effect(self.handle_square_tap(square)),
            Action::ChoosePromotion(choice) => Self::progress_effect(self.finish_promotion(choice)),
            Action::CancelPromotion => {
                self.cancel_promotion();
                Vec::new()
            }
            Action::PreviousPuzzle => Self::progress_effect(self.turn_puzzle(false)),
            Action::NextPuzzle => Self::progress_effect(self.turn_puzzle(true)),
            Action::Reset => {
                self.board.reset();
                self.reset_attempt();
                self.transient_message = None;
                Vec::new()
            }
            Action::Flip => {
                self.flipped = !self.flipped;
                Vec::new()
            }
            Action::ToggleMode => {
                self.toggle_mode();
                Vec::new()
            }
            Action::ToggleOrientationLock => {
                self.orientation_locked = !self.orientation_locked;
                Vec::new()
            }
            Action::ToggleDescription => {
                self.description_visible = !self.description_visible;
                Vec::new()
            }
            Action::ActivateCollection(collection) => {
                let changed = self.activate_collection(collection);
                self.collection_picker_open = false;
                Self::progress_effect(changed)
            }
            Action::SetTransientMessage(message) => {
                self.transient_message = message;
                Vec::new()
            }
        }
    }

    fn progress_effect(changed: bool) -> Vec<Effect> {
        if changed {
            vec![Effect::ProgressChanged]
        } else {
            Vec::new()
        }
    }

    fn open_collection_picker(&mut self) {
        if self.collection_entries.is_empty() || self.pending_promotion.is_some() {
            return;
        }

        self.collection_picker_open = true;
        self.collection_picker_page = self
            .collection_entries
            .iter()
            .position(|entry| entry.key() == self.active_collection.key())
            .map(|index| index / COLLECTIONS_PER_PAGE)
            .unwrap_or(0);
    }

    fn request_collection(&self, index: usize) -> Vec<Effect> {
        if !self.collection_picker_open || !self.collection_picker_visible_range().contains(&index)
        {
            return Vec::new();
        }

        self.collection_entries
            .get(index)
            .map(|entry| vec![Effect::CollectionRequested(entry.key().to_owned())])
            .unwrap_or_default()
    }

    fn handle_square_tap(&mut self, square: usize) -> bool {
        if (self.mode == BoardMode::Solution && self.feedback == SolutionFeedback::Complete)
            || self.pending_promotion.is_some()
        {
            return false;
        }

        let before = self.board.clone();
        match self.board.tap(square) {
            TapResult::NoChange | TapResult::SelectionChanged => false,
            TapResult::Moved { from, to } => {
                if self.mode == BoardMode::Solution {
                    let attempted =
                        UciMove::new(from, to, None).expect("board tap returns distinct squares");
                    self.check_solver_move(before, attempted)
                } else {
                    false
                }
            }
            TapResult::Promotion { from, to, color } => {
                self.pending_promotion = Some(PendingPromotion {
                    before,
                    from,
                    to,
                    color,
                });
                false
            }
        }
    }

    fn finish_promotion(&mut self, choice: PromotionChoice) -> bool {
        let Some(pending) = self.pending_promotion.take() else {
            return false;
        };

        let before = pending.before;
        self.board = before.clone();
        self.board.clear_selection();
        let kind = choice.piece_kind();
        if !self.board.promote_pawn(pending.from, pending.to, kind) {
            self.board = before;
            self.board.clear_selection();
            self.transient_message =
                Some("The staged pawn promotion could not be applied.".to_owned());
            return false;
        }

        if self.mode == BoardMode::Solution {
            let attempted = UciMove::new(pending.from, pending.to, Some(kind))
                .expect("promotion choice produces valid UCI");
            self.check_solver_move(before, attempted)
        } else {
            false
        }
    }

    fn cancel_promotion(&mut self) {
        let Some(pending) = self.pending_promotion.take() else {
            return;
        };
        self.board = pending.before;
        self.board.clear_selection();
    }

    fn check_solver_move(&mut self, before: Board, attempted: UciMove) -> bool {
        let original_ply = self.solution_ply;
        let expected = self.active_puzzle().solution.get(original_ply).cloned();
        let Some(expected) = expected else {
            self.restore_wrong_move(before);
            return false;
        };
        let Ok(expected) = parse_uci_move(&expected) else {
            self.restore_invalid_stored_move(before, original_ply, &expected);
            return false;
        };

        if attempted != expected {
            self.restore_wrong_move(before);
            return false;
        }

        self.solution_ply += 1;
        let reply = self
            .active_puzzle()
            .solution
            .get(self.solution_ply)
            .cloned();
        if let Some(reply) = reply {
            let Ok(parsed) = parse_uci_move(&reply) else {
                self.restore_invalid_stored_move(before, original_ply, &reply);
                return false;
            };
            if !self.board.apply_uci_move(parsed) {
                self.restore_invalid_stored_move(before, original_ply, &reply);
                return false;
            }
            self.solution_ply += 1;
        }

        let complete = self.solution_ply >= self.active_puzzle().solution.len();
        self.feedback = if complete {
            SolutionFeedback::Complete
        } else {
            SolutionFeedback::Correct
        };
        if complete {
            self.description_visible = true;
            return self.mark_current_solved();
        }
        false
    }

    fn restore_wrong_move(&mut self, before: Board) {
        self.board = before;
        self.board.clear_selection();
        self.feedback = SolutionFeedback::Wrong;
    }

    fn restore_invalid_stored_move(&mut self, before: Board, original_ply: usize, movement: &str) {
        self.board = before;
        self.board.clear_selection();
        self.solution_ply = original_ply;
        self.feedback = SolutionFeedback::None;
        let puzzle_id = self.active_puzzle().id.clone();
        self.transient_message = Some(format!(
            "Puzzle {puzzle_id}: stored move {movement} could not be applied."
        ));
    }

    fn toggle_mode(&mut self) {
        self.pending_promotion = None;
        self.description_visible = false;
        self.board.clear_selection();
        match self.mode {
            BoardMode::Solution => {
                self.mode = BoardMode::FreeBoard;
                self.feedback = SolutionFeedback::None;
            }
            BoardMode::FreeBoard => {
                self.board.reset();
                self.reset_attempt();
                self.mode = BoardMode::Solution;
            }
        }
    }

    fn turn_puzzle(&mut self, forward: bool) -> bool {
        let next = if forward {
            self.puzzle_index
                .checked_add(1)
                .filter(|&index| index < self.active_collection.puzzles.len())
        } else {
            self.puzzle_index.checked_sub(1)
        };
        let Some(next) = next else {
            return false;
        };

        self.select_puzzle(next);
        self.remember_current_puzzle()
    }

    fn activate_collection(&mut self, collection: ActiveCollection) -> bool {
        let mut progress_changed = self.remember_current_puzzle();
        let puzzle_index = remembered_puzzle_index(&collection, &self.progress);
        self.active_collection = collection;
        self.select_puzzle(puzzle_index);

        let key = self.active_collection.key.clone();
        let puzzle_id = self.active_puzzle().id.clone();
        progress_changed |= self.progress.set_active_file(&key);
        progress_changed |= self.progress.remember_puzzle(&key, &puzzle_id);
        progress_changed
    }

    fn select_puzzle(&mut self, index: usize) {
        let puzzle = &self.active_collection.puzzles[index];
        self.board = Board::from_fen(&puzzle.fen).expect("parsed puzzle FEN remains valid");
        if !self.orientation_locked {
            self.flipped = puzzle.side_to_move() == Color::Black;
        }
        self.puzzle_index = index;
        self.reset_attempt();
        self.transient_message = None;
    }

    fn reset_attempt(&mut self) {
        self.description_visible = false;
        self.solution_ply = 0;
        self.feedback = SolutionFeedback::None;
        self.pending_promotion = None;
        self.board.clear_selection();
    }

    fn remember_current_puzzle(&mut self) -> bool {
        let key = self.active_collection.key.clone();
        let puzzle_id = self.active_puzzle().id.clone();
        self.progress.remember_puzzle(&key, &puzzle_id)
    }

    fn mark_current_solved(&mut self) -> bool {
        let key = self.active_collection.key.clone();
        let puzzle_id = self.active_puzzle().id.clone();
        self.progress.mark_solved(&key, &puzzle_id)
    }
}

fn remembered_puzzle_index(collection: &ActiveCollection, progress: &Progress) -> usize {
    progress
        .file(collection.key())
        .and_then(|file| file.current_puzzle_id.as_deref())
        .and_then(|id| collection.puzzles.iter().position(|puzzle| puzzle.id == id))
        .unwrap_or(0)
}
