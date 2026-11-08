//! Transitional combined composition; puzzle behavior lives in puzzle_app.
pub use crate::interaction::{PendingPromotion, PromotionChoice};
pub use crate::puzzle_app::{
    ActiveCollection, AnalysisBrowserState, BoardMode, SolutionFeedback, COLLECTIONS_PER_PAGE,
};
use crate::puzzle_app::{PuzzleAction, PuzzleEffect, PuzzleState};
use crate::review_app::{ReviewAction, ReviewApp, ReviewEffect};
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
    review: ReviewApp,
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
            review: ReviewApp::new(
                Vec::new(),
                settings.for_workspace(Workspace::Review).clone(),
            ),
            settings,
            workspace: Workspace::Puzzles,
        }
    }
    pub const fn workspace(&self) -> Workspace {
        self.workspace
    }
    pub fn set_review_games(&mut self, games: Vec<ReviewGameEntry>) {
        self.review.set_review_games(games);
        if self.review.review_state().is_none() && self.workspace == Workspace::Review {
            self.workspace = Workspace::Puzzles;
            self.puzzle.dispatch(PuzzleAction::SetTransientMessage(
                self.review.transient_message().map(str::to_owned),
            ));
        }
    }
    pub fn review_resume(&self) -> Option<crate::ReviewResume> {
        self.review.review_resume()
    }
    pub fn restore_review_resume(&mut self, resume: &crate::ReviewResume) -> Option<String> {
        self.review.restore_review_resume(resume)
    }
    pub fn review_games(&self) -> &[ReviewGameEntry] {
        self.review.review_games()
    }
    pub fn set_review_file_errors(&mut self, errors: Vec<ReviewFileError>) {
        self.review.set_review_file_errors(errors)
    }
    pub fn review_file_errors(&self) -> &[ReviewFileError] {
        self.review.review_file_errors()
    }
    pub fn review_picker_entry_count(&self) -> usize {
        self.review.review_picker_entry_count()
    }
    pub fn review_picker_game(&self, picker_index: usize) -> Option<&ReviewGameEntry> {
        self.review.review_picker_game(picker_index)
    }
    pub fn review_picker_error(&self, picker_index: usize) -> Option<&ReviewFileError> {
        self.review.review_picker_error(picker_index)
    }
    pub fn review_state(&self) -> Option<&ReviewState> {
        self.review.review_state()
    }
    pub fn active_review_game(&self) -> Option<&ReviewGameEntry> {
        self.review.active_review_game()
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
        match self.workspace {
            Workspace::Puzzles => self.puzzle.board(),
            Workspace::Review => self.review.board().unwrap_or_else(|| self.puzzle.board()),
        }
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
            Workspace::Puzzles => self.puzzle.selected_analysis_node(),
            Workspace::Review => self.review.review_state().map(ReviewState::selected_node),
        }
    }
    pub fn analysis_page(&self) -> usize {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.analysis_page(),
            Workspace::Review => self
                .review
                .review_state()
                .map_or(0, ReviewState::analysis_page),
        }
    }
    pub fn analysis_can_previous_page(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.analysis_can_previous_page(),
            Workspace::Review => self
                .review
                .review_state()
                .is_some_and(|state| state.analysis_page() > 0),
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
            Workspace::Puzzles => self.puzzle.pending_promotion(),
            Workspace::Review => self.review.pending_promotion(),
        }
    }
    pub fn flipped(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.flipped(),
            Workspace::Review => self.review.review_state().is_some_and(ReviewState::flipped),
        }
    }
    pub fn orientation_locked(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.orientation_locked(),
            Workspace::Review => self
                .review
                .review_state()
                .is_some_and(ReviewState::orientation_locked),
        }
    }
    pub const fn description_visible(&self) -> bool {
        self.puzzle.description_visible()
    }
    pub const fn settings(&self) -> &WorkspaceSettings {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.settings(),
            Workspace::Review => self.review.settings(),
        }
    }
    pub const fn persisted_settings(&self) -> &Settings {
        &self.settings
    }
    pub const fn settings_open(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.settings_open(),
            Workspace::Review => self.review.settings_open(),
        }
    }
    pub const fn progress(&self) -> &Progress {
        self.puzzle.progress()
    }
    pub fn transient_message(&self) -> Option<&str> {
        match self.workspace {
            Workspace::Puzzles => self.puzzle.transient_message(),
            Workspace::Review => self.review.transient_message(),
        }
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
        self.workspace == Workspace::Puzzles && self.puzzle.can_send_position()
    }
    pub const fn physical_board_seat(&self) -> crate::PhysicalBoardSeat {
        self.puzzle.physical_board_seat()
    }
    pub fn board_input_status(&self) -> crate::BoardInputStatus {
        self.puzzle.board_input_status()
    }

    fn sync_puzzle_settings(&mut self) {
        *self.settings.for_workspace_mut(Workspace::Puzzles) = self.puzzle.settings().clone();
        self.settings
            .set_physical_board_seat(self.puzzle.physical_board_seat());
    }
    pub fn dispatch(&mut self, action: Action) -> Vec<Effect> {
        if matches!(action, Action::BoardStatus(_) | Action::PauseBoardInput) {
            return self.dispatch_puzzle(action);
        }
        if self.workspace == Workspace::Puzzles && !matches!(action, Action::ToggleWorkspace) {
            return self.dispatch_puzzle(action);
        }
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
        if matches!(action, Action::ToggleWorkspace) {
            if self.settings_open()
                || self.puzzle.puzzle_goto_open()
                || self.puzzle.collection_picker_open()
                || self
                    .review
                    .review_state()
                    .is_some_and(ReviewState::game_picker_open)
            {
                return Vec::new();
            }
            match self.workspace {
                Workspace::Puzzles if self.review.review_state().is_some() => {
                    self.puzzle
                        .dispatch(PuzzleAction::SetTransientMessage(None));
                    self.review
                        .dispatch(ReviewAction::SetTransientMessage(None));
                    self.workspace = Workspace::Review;
                }
                Workspace::Puzzles => {
                    self.puzzle.dispatch(PuzzleAction::SetTransientMessage(Some(
                        "No review games are available.".to_owned(),
                    )));
                }
                Workspace::Review => {
                    self.puzzle.dispatch(PuzzleAction::SetTransientMessage(
                        self.review.transient_message().map(str::to_owned),
                    ));
                    self.workspace = Workspace::Puzzles;
                }
            }
            return Vec::new();
        }
        let Some(action) = review_action(action) else {
            return Vec::new();
        };
        let effects = self.review.dispatch(action);
        *self.settings.for_workspace_mut(Workspace::Review) = self.review.settings().clone();
        effects
            .into_iter()
            .map(|effect| match effect {
                ReviewEffect::ExitRequested => Effect::ExitRequested,
                ReviewEffect::SettingsChanged => Effect::SettingsChanged,
                ReviewEffect::ReviewGameRequested(key) => Effect::ReviewGameRequested(key),
            })
            .collect()
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

fn review_action(action: Action) -> Option<ReviewAction> {
    Some(match action {
        Action::Exit => ReviewAction::Exit,
        Action::ReviewPrevious => ReviewAction::ReviewPrevious,
        Action::ReviewNext => ReviewAction::ReviewNext,
        Action::ReviewReset => ReviewAction::ReviewReset,
        Action::ReviewFlip => ReviewAction::ReviewFlip,
        Action::ReviewToggleFree => ReviewAction::ReviewToggleFree,
        Action::ReviewToggleOrientationLock => ReviewAction::ReviewToggleOrientationLock,
        Action::OpenReviewGamePicker => ReviewAction::OpenReviewGamePicker,
        Action::CloseReviewGamePicker => ReviewAction::CloseReviewGamePicker,
        Action::ReviewGamePickerPreviousPage => ReviewAction::ReviewGamePickerPreviousPage,
        Action::ReviewGamePickerNextPage => ReviewAction::ReviewGamePickerNextPage,
        Action::SelectReviewGame(value) => ReviewAction::SelectReviewGame(value),
        Action::ActivateReviewGame(key, game) => ReviewAction::ActivateReviewGame(key, game),
        Action::TapSquare(value) => ReviewAction::TapSquare(value),
        Action::ChoosePromotion(value) => ReviewAction::ChoosePromotion(value),
        Action::CancelPromotion => ReviewAction::CancelPromotion,
        Action::SelectAnalysisNode(value) => ReviewAction::SelectAnalysisNode(value),
        Action::AnalysisPreviousPage => ReviewAction::AnalysisPreviousPage,
        Action::AnalysisNextPage => ReviewAction::AnalysisNextPage,
        Action::OpenSettings => ReviewAction::OpenSettings,
        Action::CloseSettings => ReviewAction::CloseSettings,
        Action::ToggleFreeModeSetting => ReviewAction::ToggleFreeModeSetting,
        Action::ToggleNotesSetting => ReviewAction::ToggleNotesSetting,
        Action::ToggleLockSetting => ReviewAction::ToggleLockSetting,
        Action::ToggleFlipSetting => ReviewAction::ToggleFlipSetting,
        Action::ToggleBoardSizeSetting => ReviewAction::ToggleBoardSizeSetting,
        Action::SetTransientMessage(value) => ReviewAction::SetTransientMessage(value),
        _ => return None,
    })
}
