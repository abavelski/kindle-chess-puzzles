//! Transitional combined composition; puzzle behavior lives in puzzle_app.
pub use crate::interaction::{PendingPromotion, PromotionChoice};
pub use crate::puzzle_app::{
    ActiveCollection, AnalysisBrowserState, BoardMode, SolutionFeedback, COLLECTIONS_PER_PAGE,
};
use crate::puzzle_app::{PuzzleAction, PuzzleEffect, PuzzleState};
use crate::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    ToggleBoardConnection,
    SendBoardPosition,
    BoardStatus(BoardSessionStatus),
    BoardPosition(String),
    PauseBoardInput,
    Exit,
    ToggleWorkspace,
    ReviewPrevious,
    ReviewNext,
    ReviewReset,
    ReviewFlip,
    ReviewToggleFree,
    ReviewToggleOrientationLock,
    OpenReviewGamePicker,
    CloseReviewGamePicker,
    ReviewGamePickerPreviousPage,
    ReviewGamePickerNextPage,
    SelectReviewGame(usize),
    ActivateReviewGame(ReviewGameKey, Box<ReviewGame>),
    TapSquare(usize),
    ChoosePromotion(PromotionChoice),
    CancelPromotion,
    ToggleAnalysis,
    OpenAnalysis,
    CloseAnalysis,
    SelectAnalysisNode(AnalysisNodeIndex),
    AnalysisPreviousPage,
    AnalysisNextPage,
    OpenPuzzleGoto,
    PuzzleGotoDigit(u8),
    PuzzleGotoBackspace,
    ConfirmPuzzleGoto,
    CancelPuzzleGoto,
    PreviousPuzzle,
    NextPuzzle,
    Reset,
    Flip,
    ToggleMode,
    ToggleOrientationLock,
    ToggleDescription,
    OpenSettings,
    CloseSettings,
    ToggleFreeModeSetting,
    ToggleNotesSetting,
    ToggleLockSetting,
    ToggleFlipSetting,
    ToggleBoardSizeSetting,
    CyclePhysicalBoardSeat,
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
    Board(BoardIntent),
    ExitRequested,
    ProgressChanged,
    SettingsChanged,
    CollectionRequested(String),
    ReviewGameRequested(ReviewGameKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    puzzle: PuzzleState,
    settings: Settings,
    workspace: Workspace,
    review_games: Vec<ReviewGameEntry>,
    review_file_errors: Vec<ReviewFileError>,
    review_state: Option<ReviewState>,
}
impl AppState {
    pub fn new(active_collection: ActiveCollection, progress: Progress) -> Self {
        Self::new_with_settings(active_collection, progress, Settings::default())
    }
    pub fn new_with_settings(
        active_collection: ActiveCollection,
        progress: Progress,
        settings: Settings,
    ) -> Self {
        Self {
            puzzle: PuzzleState::new_with_settings(
                active_collection,
                progress,
                settings.for_workspace(Workspace::Puzzles).clone(),
                settings.physical_board_seat(),
            ),
            settings,
            workspace: Workspace::Puzzles,
            review_games: Vec::new(),
            review_file_errors: Vec::new(),
            review_state: None,
        }
    }
    pub const fn workspace(&self) -> Workspace {
        self.workspace
    }
    pub fn set_review_games(&mut self, games: Vec<ReviewGameEntry>) {
        let previous_key = self.active_review_game().map(|entry| entry.key().clone());
        self.review_games = games;

        if self.review_games.is_empty() {
            self.review_state = None;
            if self.workspace == Workspace::Review {
                self.workspace = Workspace::Puzzles;
                self.puzzle.transient_message = Some("No review games are available.".to_owned());
            }
            return;
        }

        let active_game_index = previous_key
            .as_ref()
            .and_then(|key| {
                self.review_games
                    .iter()
                    .position(|entry| entry.key() == key)
            })
            .unwrap_or(0);
        self.review_state = Some(ReviewState::new(
            active_game_index,
            self.review_games[active_game_index].game(),
        ));
    }
    pub fn review_resume(&self) -> Option<crate::ReviewResume> {
        let entry = self.active_review_game()?;
        Some(crate::ReviewResume::capture(
            entry.key().collection_id(),
            entry.game(),
            self.review_state.as_ref()?,
        ))
    }
    pub fn restore_review_resume(&mut self, resume: &crate::ReviewResume) -> Option<String> {
        let Some(index) = self.review_games.iter().position(|entry| {
            entry.key().collection_id() == resume.collection_id
                && entry.key().game_id() == resume.game_id
        }) else {
            return Some(
                "Remembered review game is missing or invalid; using the first available game."
                    .into(),
            );
        };
        let game = self.review_games[index].game();
        let mut state = ReviewState::new(index, game);
        let warning = if let Some(node) = resume.selected_node(game) {
            state.select_node(game, node);
            state.restore_preferences(resume);
            None
        } else {
            Some("Remembered review move has changed or is missing; using the game starting position.".into())
        };
        self.review_state = Some(state);
        warning
    }
    pub fn review_games(&self) -> &[ReviewGameEntry] {
        &self.review_games
    }
    pub fn set_review_file_errors(&mut self, errors: Vec<ReviewFileError>) {
        self.review_file_errors = errors;
    }
    pub fn review_file_errors(&self) -> &[ReviewFileError] {
        &self.review_file_errors
    }
    pub fn review_picker_entry_count(&self) -> usize {
        self.review_games.len() + self.review_file_errors.len()
    }
    pub fn review_picker_game(&self, picker_index: usize) -> Option<&ReviewGameEntry> {
        self.review_games.get(picker_index)
    }
    pub fn review_picker_error(&self, picker_index: usize) -> Option<&ReviewFileError> {
        picker_index
            .checked_sub(self.review_games.len())
            .and_then(|index| self.review_file_errors.get(index))
    }
    pub fn review_state(&self) -> Option<&ReviewState> {
        self.review_state.as_ref()
    }
    pub fn active_review_game(&self) -> Option<&ReviewGameEntry> {
        let index = self.review_state.as_ref()?.active_game_index();
        self.review_games.get(index)
    }
    pub fn active_collection(&self) -> &ActiveCollection {
        self.puzzle.active_collection()
    }
    pub const fn active_puzzle_index(&self) -> usize {
        self.puzzle.active_puzzle_index()
    }
    pub const fn can_previous_puzzle(&self) -> bool {
        self.puzzle.can_previous_puzzle()
    }
    pub fn can_next_puzzle(&self) -> bool {
        self.puzzle.can_next_puzzle()
    }
    pub const fn puzzle_goto_open(&self) -> bool {
        self.puzzle.puzzle_goto_open()
    }
    pub fn puzzle_goto_input(&self) -> Option<&str> {
        self.puzzle.puzzle_goto_input()
    }
    pub fn puzzle_goto_number(&self) -> Option<usize> {
        self.puzzle.puzzle_goto_number()
    }
    pub fn active_puzzle(&self) -> &Puzzle {
        self.puzzle.active_puzzle()
    }
    pub fn board(&self) -> &Board {
        if self.workspace == Workspace::Review {
            if let Some(review) = self.review_state.as_ref() {
                return review.board();
            }
        }
        self.puzzle
            .analysis_preview_board
            .as_ref()
            .unwrap_or(&self.puzzle.board)
    }
    pub const fn live_board(&self) -> &Board {
        self.puzzle.live_board()
    }
    pub fn analysis_available(&self) -> bool {
        self.puzzle.analysis_available()
    }
    pub const fn analysis_browser_open(&self) -> bool {
        self.puzzle.analysis_browser_open()
    }
    pub fn analysis_browser(&self) -> Option<&AnalysisBrowserState> {
        self.puzzle.analysis_browser()
    }
    pub fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        match self.workspace {
            Workspace::Puzzles => self
                .puzzle
                .analysis_browser
                .as_ref()
                .and_then(|browser| browser.selected_node()),
            Workspace::Review => self.review_state.as_ref().map(ReviewState::selected_node),
        }
    }
    pub fn analysis_page(&self) -> usize {
        match self.workspace {
            Workspace::Puzzles => self
                .puzzle
                .analysis_browser
                .as_ref()
                .map_or(0, AnalysisBrowserState::page),
            Workspace::Review => self
                .review_state
                .as_ref()
                .map_or(0, ReviewState::analysis_page),
        }
    }
    pub fn analysis_can_previous_page(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self
                .puzzle
                .analysis_browser
                .as_ref()
                .is_some_and(|browser| browser.page() > 0),
            Workspace::Review => self
                .review_state
                .as_ref()
                .is_some_and(|review| review.analysis_page() > 0),
        }
    }
    pub const fn mode(&self) -> BoardMode {
        self.puzzle.mode()
    }
    pub const fn solution_ply(&self) -> usize {
        self.puzzle.solution_ply()
    }
    pub const fn feedback(&self) -> SolutionFeedback {
        self.puzzle.feedback()
    }
    pub fn pending_promotion(&self) -> Option<&PendingPromotion> {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.pending_promotion.as_ref(),
            Workspace::Review => self
                .review_state
                .as_ref()
                .and_then(ReviewState::pending_promotion),
        }
    }
    pub fn flipped(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.flipped,
            Workspace::Review => self.review_state.as_ref().is_some_and(ReviewState::flipped),
        }
    }
    pub fn orientation_locked(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.orientation_locked,
            Workspace::Review => self
                .review_state
                .as_ref()
                .is_some_and(ReviewState::orientation_locked),
        }
    }
    pub const fn description_visible(&self) -> bool {
        self.puzzle.description_visible()
    }
    pub const fn settings(&self) -> &WorkspaceSettings {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.settings(),
            Workspace::Review => self.settings.for_workspace(Workspace::Review),
        }
    }
    pub const fn persisted_settings(&self) -> &Settings {
        &self.settings
    }
    pub const fn settings_open(&self) -> bool {
        self.puzzle.settings_open()
    }
    pub const fn progress(&self) -> &Progress {
        self.puzzle.progress()
    }
    pub fn transient_message(&self) -> Option<&str> {
        self.puzzle.transient_message()
    }
    pub fn is_current_solved(&self) -> bool {
        self.puzzle.is_current_solved()
    }
    pub fn set_collection_entries(&mut self, entries: Vec<CollectionEntry>) {
        self.puzzle.set_collection_entries(entries)
    }
    pub fn collection_entries(&self) -> &[CollectionEntry] {
        self.puzzle.collection_entries()
    }
    pub const fn collection_picker_open(&self) -> bool {
        self.puzzle.collection_picker_open()
    }
    pub const fn collection_picker_page(&self) -> usize {
        self.puzzle.collection_picker_page()
    }
    pub fn collection_picker_page_count(&self) -> usize {
        self.puzzle.collection_picker_page_count()
    }
    pub fn collection_picker_visible_range(&self) -> std::ops::Range<usize> {
        self.puzzle.collection_picker_visible_range()
    }
    pub const fn collection_picker_can_previous_page(&self) -> bool {
        self.puzzle.collection_picker_can_previous_page()
    }
    pub fn collection_picker_can_next_page(&self) -> bool {
        self.puzzle.collection_picker_can_next_page()
    }
    pub fn board_status(&self) -> &BoardSessionStatus {
        self.puzzle.board_status()
    }
    pub fn board_target(&self) -> Option<&str> {
        self.puzzle.board_target()
    }
    pub fn can_send_position(&self) -> bool {
        self.workspace == Workspace::Puzzles
            && self.puzzle.board_status.idle()
            && self.puzzle.pending_promotion().is_none()
            && !self.puzzle.settings_open
            && !self.puzzle.collection_picker_open
            && !self.puzzle_goto_open()
    }
    pub const fn physical_board_seat(&self) -> crate::PhysicalBoardSeat {
        self.puzzle.physical_board_seat()
    }
    pub fn board_input_status(&self) -> crate::BoardInputStatus {
        self.puzzle.board_input_status()
    }
    fn toggle_workspace(&mut self) {
        match self.workspace {
            Workspace::Puzzles => {
                if self.review_state.is_some() {
                    self.workspace = Workspace::Review;
                    self.puzzle.transient_message = None;
                } else {
                    self.puzzle.transient_message =
                        Some("No review games are available.".to_owned());
                }
            }
            Workspace::Review => {
                self.workspace = Workspace::Puzzles;
            }
        }
    }
    fn navigate_review(&mut self, forward: bool) {
        let (games, review_state) = (&self.review_games, &mut self.review_state);
        let Some(review) = review_state.as_mut() else {
            return;
        };
        let Some(entry) = games.get(review.active_game_index()) else {
            return;
        };
        review.navigate_main_line(entry.game(), forward);
    }
    fn select_review_node(&mut self, index: AnalysisNodeIndex) {
        let (games, review_state) = (&self.review_games, &mut self.review_state);
        let Some(review) = review_state.as_mut() else {
            return;
        };
        let Some(entry) = games.get(review.active_game_index()) else {
            return;
        };
        review.select_node(entry.game(), index);
    }
    fn open_review_game_picker(&mut self) {
        let total = self.review_picker_entry_count();
        if let Some(review) = self.review_state.as_mut() {
            review.open_game_picker(total);
        }
    }
    fn request_review_game(&self, index: usize) -> Vec<Effect> {
        let total = self.review_picker_entry_count();
        let Some(review) = self.review_state.as_ref() else {
            return Vec::new();
        };
        if !review.game_picker_open() || !review.game_picker_visible_range(total).contains(&index) {
            return Vec::new();
        }

        self.review_games
            .get(index)
            .map(|entry| vec![Effect::ReviewGameRequested(entry.key().clone())])
            .unwrap_or_default()
    }
    fn activate_review_game(&mut self, key: &ReviewGameKey, game: ReviewGame) {
        let total = self.review_picker_entry_count();
        let (games, review_state) = (&mut self.review_games, &mut self.review_state);
        let Some(review) = review_state.as_mut() else {
            return;
        };
        let Some(index) = games.iter().position(|entry| entry.key() == key) else {
            return;
        };
        if !review.game_picker_open() || !review.game_picker_visible_range(total).contains(&index) {
            return;
        }

        let replacement = ReviewGameEntry::new(key.collection_id().to_owned(), game);
        if replacement.key() != key {
            return;
        }
        games[index] = replacement;
        review.activate_game(index, games[index].game());
    }

    fn sync_puzzle_settings(&mut self) {
        *self.settings.for_workspace_mut(Workspace::Puzzles) = self.puzzle.settings().clone();
        self.settings
            .set_physical_board_seat(self.puzzle.physical_board_seat());
    }
    pub fn dispatch(&mut self, action: Action) -> Vec<Effect> {
        // Global board transport status must reach Puzzles even while Review is visible.
        if matches!(action, Action::BoardStatus(_) | Action::PauseBoardInput) {
            return self.dispatch_puzzle(action);
        }
        if self.workspace == Workspace::Puzzles && !matches!(action, Action::ToggleWorkspace) {
            return self.dispatch_puzzle(action);
        }
        // Switching and review interactions invalidate physical input as before.
        if matches!(
            action,
            Action::ToggleWorkspace
                | Action::TapSquare(_)
                | Action::ChoosePromotion(_)
                | Action::CancelPromotion
                | Action::SelectAnalysisNode(_)
        ) {
            self.puzzle.dispatch(PuzzleAction::PauseBoardInput);
        }
        if self.puzzle.settings_open
            && !matches!(
                action,
                Action::Exit
                    | Action::CloseSettings
                    | Action::ToggleFreeModeSetting
                    | Action::ToggleNotesSetting
                    | Action::ToggleLockSetting
                    | Action::ToggleFlipSetting
                    | Action::ToggleBoardSizeSetting
                    | Action::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }
        if self.puzzle.puzzle_goto_open() || self.puzzle.collection_picker_open() {
            return Vec::new();
        }
        if self
            .review_state
            .as_ref()
            .is_some_and(ReviewState::game_picker_open)
            && !matches!(
                action,
                Action::Exit
                    | Action::CloseReviewGamePicker
                    | Action::ReviewGamePickerPreviousPage
                    | Action::ReviewGamePickerNextPage
                    | Action::SelectReviewGame(_)
                    | Action::ActivateReviewGame(_, _)
                    | Action::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }
        match action {
            Action::Exit => vec![Effect::ExitRequested],
            Action::ToggleWorkspace => {
                self.toggle_workspace();
                Vec::new()
            }
            Action::OpenSettings => {
                if self.pending_promotion().is_none() {
                    self.puzzle.settings_open = true;
                }
                Vec::new()
            }
            Action::CloseSettings => {
                self.puzzle.settings_open = false;
                Vec::new()
            }
            Action::SetTransientMessage(message) => {
                self.puzzle.transient_message = message;
                Vec::new()
            }
            Action::ToggleFreeModeSetting => {
                let enabled = !self.settings().show_free_mode_button();
                self.settings
                    .for_workspace_mut(Workspace::Review)
                    .set_show_free_mode_button(enabled);
                if !enabled {
                    if let Some(review) = self.review_state.as_mut() {
                        review.set_free_enabled(false);
                    }
                }
                vec![Effect::SettingsChanged]
            }
            Action::ToggleNotesSetting => {
                let enabled = !self.settings().show_notes_button();
                self.settings
                    .for_workspace_mut(Workspace::Review)
                    .set_show_notes_button(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::ToggleLockSetting => {
                let enabled = !self.settings().show_lock_button();
                self.settings
                    .for_workspace_mut(Workspace::Review)
                    .set_show_lock_button(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::ToggleFlipSetting => {
                let enabled = !self.settings().show_flip_button();
                self.settings
                    .for_workspace_mut(Workspace::Review)
                    .set_show_flip_button(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::ToggleBoardSizeSetting => {
                let enabled = !self.settings().small_board();
                self.settings
                    .for_workspace_mut(Workspace::Review)
                    .set_small_board(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::TapSquare(square) => {
                if let Some(review) = self.review_state.as_mut() {
                    review.handle_square_tap(square);
                }
                Vec::new()
            }
            Action::ChoosePromotion(choice) => {
                if let Some(review) = self.review_state.as_mut() {
                    review.finish_promotion(choice.piece_kind());
                }
                Vec::new()
            }
            Action::CancelPromotion => {
                if let Some(review) = self.review_state.as_mut() {
                    review.cancel_promotion();
                }
                Vec::new()
            }
            Action::AnalysisPreviousPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.analysis_previous_page();
                }
                Vec::new()
            }
            Action::AnalysisNextPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.analysis_next_page();
                }
                Vec::new()
            }
            Action::ReviewReset => {
                if let Some(review) = self.review_state.as_mut() {
                    review.reset_scratch();
                }
                Vec::new()
            }
            Action::CloseReviewGamePicker => {
                if let Some(review) = self.review_state.as_mut() {
                    review.close_game_picker();
                }
                Vec::new()
            }
            Action::ReviewGamePickerPreviousPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.game_picker_previous_page();
                }
                Vec::new()
            }
            Action::ReviewFlip => {
                if self.settings().show_flip_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.flip();
                    }
                }
                Vec::new()
            }
            Action::ReviewToggleFree => {
                if self.settings().show_free_mode_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.toggle_free();
                    }
                }
                Vec::new()
            }
            Action::ReviewToggleOrientationLock => {
                if self.settings().show_lock_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.toggle_orientation_lock();
                    }
                }
                Vec::new()
            }
            Action::ReviewPrevious => {
                self.navigate_review(false);
                Vec::new()
            }
            Action::ReviewNext => {
                self.navigate_review(true);
                Vec::new()
            }
            Action::SelectAnalysisNode(index) => {
                self.select_review_node(index);
                Vec::new()
            }
            Action::OpenReviewGamePicker => {
                self.open_review_game_picker();
                Vec::new()
            }
            Action::ReviewGamePickerNextPage => {
                let total = self.review_picker_entry_count();
                if let Some(review) = self.review_state.as_mut() {
                    review.game_picker_next_page(total);
                }
                Vec::new()
            }
            Action::SelectReviewGame(index) => self.request_review_game(index),
            Action::ActivateReviewGame(key, game) => {
                self.activate_review_game(&key, *game);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
    fn dispatch_puzzle(&mut self, action: Action) -> Vec<Effect> {
        let Some(action) = puzzle_action(action) else {
            return Vec::new();
        };
        let effects = self.puzzle.dispatch(action);
        self.sync_puzzle_settings();
        effects
            .into_iter()
            .map(|effect| match effect {
                PuzzleEffect::Board(intent) => Effect::Board(intent),
                PuzzleEffect::ExitRequested => Effect::ExitRequested,
                PuzzleEffect::ProgressChanged => Effect::ProgressChanged,
                PuzzleEffect::SettingsChanged => Effect::SettingsChanged,
                PuzzleEffect::CollectionRequested(key) => Effect::CollectionRequested(key),
            })
            .collect()
    }
}
fn puzzle_action(action: Action) -> Option<PuzzleAction> {
    Some(match action {
        Action::ToggleBoardConnection => PuzzleAction::ToggleBoardConnection,
        Action::SendBoardPosition => PuzzleAction::SendBoardPosition,
        Action::BoardStatus(value) => PuzzleAction::BoardStatus(value),
        Action::BoardPosition(value) => PuzzleAction::BoardPosition(value),
        Action::PauseBoardInput => PuzzleAction::PauseBoardInput,
        Action::Exit => PuzzleAction::Exit,
        Action::TapSquare(value) => PuzzleAction::TapSquare(value),
        Action::ChoosePromotion(value) => PuzzleAction::ChoosePromotion(value),
        Action::CancelPromotion => PuzzleAction::CancelPromotion,
        Action::ToggleAnalysis => PuzzleAction::ToggleAnalysis,
        Action::OpenAnalysis => PuzzleAction::OpenAnalysis,
        Action::CloseAnalysis => PuzzleAction::CloseAnalysis,
        Action::SelectAnalysisNode(value) => PuzzleAction::SelectAnalysisNode(value),
        Action::AnalysisPreviousPage => PuzzleAction::AnalysisPreviousPage,
        Action::AnalysisNextPage => PuzzleAction::AnalysisNextPage,
        Action::OpenPuzzleGoto => PuzzleAction::OpenPuzzleGoto,
        Action::PuzzleGotoDigit(value) => PuzzleAction::PuzzleGotoDigit(value),
        Action::PuzzleGotoBackspace => PuzzleAction::PuzzleGotoBackspace,
        Action::ConfirmPuzzleGoto => PuzzleAction::ConfirmPuzzleGoto,
        Action::CancelPuzzleGoto => PuzzleAction::CancelPuzzleGoto,
        Action::PreviousPuzzle => PuzzleAction::PreviousPuzzle,
        Action::NextPuzzle => PuzzleAction::NextPuzzle,
        Action::Reset => PuzzleAction::Reset,
        Action::Flip => PuzzleAction::Flip,
        Action::ToggleMode => PuzzleAction::ToggleMode,
        Action::ToggleOrientationLock => PuzzleAction::ToggleOrientationLock,
        Action::ToggleDescription => PuzzleAction::ToggleDescription,
        Action::OpenSettings => PuzzleAction::OpenSettings,
        Action::CloseSettings => PuzzleAction::CloseSettings,
        Action::ToggleFreeModeSetting => PuzzleAction::ToggleFreeModeSetting,
        Action::ToggleNotesSetting => PuzzleAction::ToggleNotesSetting,
        Action::ToggleLockSetting => PuzzleAction::ToggleLockSetting,
        Action::ToggleFlipSetting => PuzzleAction::ToggleFlipSetting,
        Action::ToggleBoardSizeSetting => PuzzleAction::ToggleBoardSizeSetting,
        Action::CyclePhysicalBoardSeat => PuzzleAction::CyclePhysicalBoardSeat,
        Action::OpenCollectionPicker => PuzzleAction::OpenCollectionPicker,
        Action::CloseCollectionPicker => PuzzleAction::CloseCollectionPicker,
        Action::CollectionPickerPreviousPage => PuzzleAction::CollectionPickerPreviousPage,
        Action::CollectionPickerNextPage => PuzzleAction::CollectionPickerNextPage,
        Action::SelectCollection(value) => PuzzleAction::SelectCollection(value),
        Action::ActivateCollection(value) => PuzzleAction::ActivateCollection(value),
        Action::SetTransientMessage(value) => PuzzleAction::SetTransientMessage(value),
        _ => return None,
    })
}
