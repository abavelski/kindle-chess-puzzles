//! Review-only library, navigation, scratch board, settings and resume effects.
//!
//! Review has no grading/progress effect:
//! ```compile_fail
//! use chess_core::review_app::ReviewEffect;
//! let effect = ReviewEffect::ProgressChanged;
//! ```
use crate::{
    AnalysisNodeIndex, Board, PendingPromotion, PromotionChoice, ReviewFileError, ReviewGame,
    ReviewGameEntry, ReviewGameKey, ReviewState, WorkspaceSettings,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewAction {
    Exit,
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
    SelectAnalysisNode(AnalysisNodeIndex),
    AnalysisPreviousPage,
    AnalysisNextPage,
    OpenSettings,
    CloseSettings,
    ToggleFreeModeSetting,
    ToggleNotesSetting,
    ToggleLockSetting,
    ToggleFlipSetting,
    ToggleBoardSizeSetting,
    SetTransientMessage(Option<String>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewEffect {
    ExitRequested,
    SettingsChanged,
    ReviewGameRequested(ReviewGameKey),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewApp {
    review_games: Vec<ReviewGameEntry>,
    review_file_errors: Vec<ReviewFileError>,
    review_state: Option<ReviewState>,
    settings: WorkspaceSettings,
    settings_open: bool,
    transient_message: Option<String>,
}
impl ReviewApp {
    pub fn new(games: Vec<ReviewGameEntry>, settings: WorkspaceSettings) -> Self {
        let mut app = Self {
            review_games: Vec::new(),
            review_file_errors: Vec::new(),
            review_state: None,
            settings,
            settings_open: false,
            transient_message: None,
        };
        app.set_review_games(games);
        app
    }
    pub const fn settings(&self) -> &WorkspaceSettings {
        &self.settings
    }
    pub const fn settings_open(&self) -> bool {
        self.settings_open
    }
    pub fn transient_message(&self) -> Option<&str> {
        self.transient_message.as_deref()
    }
    pub fn board(&self) -> Option<&Board> {
        self.review_state.as_ref().map(ReviewState::board)
    }
    pub fn pending_promotion(&self) -> Option<&PendingPromotion> {
        self.review_state
            .as_ref()
            .and_then(ReviewState::pending_promotion)
    }
    pub fn set_review_games(&mut self, games: Vec<ReviewGameEntry>) {
        let previous_key = self.active_review_game().map(|entry| entry.key().clone());
        self.review_games = games;

        if self.review_games.is_empty() {
            self.review_state = None;
            self.transient_message = Some("No review games are available.".to_owned());
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
    fn request_review_game(&self, index: usize) -> Vec<ReviewEffect> {
        let total = self.review_picker_entry_count();
        let Some(review) = self.review_state.as_ref() else {
            return Vec::new();
        };
        if !review.game_picker_open() || !review.game_picker_visible_range(total).contains(&index) {
            return Vec::new();
        }

        self.review_games
            .get(index)
            .map(|entry| vec![ReviewEffect::ReviewGameRequested(entry.key().clone())])
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
    pub fn dispatch(&mut self, action: ReviewAction) -> Vec<ReviewEffect> {
        if self.settings_open
            && !matches!(
                action,
                ReviewAction::Exit
                    | ReviewAction::CloseSettings
                    | ReviewAction::ToggleFreeModeSetting
                    | ReviewAction::ToggleNotesSetting
                    | ReviewAction::ToggleLockSetting
                    | ReviewAction::ToggleFlipSetting
                    | ReviewAction::ToggleBoardSizeSetting
                    | ReviewAction::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }
        if self
            .review_state
            .as_ref()
            .is_some_and(ReviewState::game_picker_open)
            && !matches!(
                action,
                ReviewAction::Exit
                    | ReviewAction::CloseReviewGamePicker
                    | ReviewAction::ReviewGamePickerPreviousPage
                    | ReviewAction::ReviewGamePickerNextPage
                    | ReviewAction::SelectReviewGame(_)
                    | ReviewAction::ActivateReviewGame(_, _)
                    | ReviewAction::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }
        match action {
            ReviewAction::Exit => vec![ReviewEffect::ExitRequested],
            ReviewAction::OpenSettings => {
                if self.pending_promotion().is_none() {
                    self.settings_open = true;
                }
                Vec::new()
            }
            ReviewAction::CloseSettings => {
                self.settings_open = false;
                Vec::new()
            }
            ReviewAction::SetTransientMessage(message) => {
                self.transient_message = message;
                Vec::new()
            }
            ReviewAction::ToggleFreeModeSetting => {
                let enabled = !self.settings().show_free_mode_button();
                self.settings.set_show_free_mode_button(enabled);
                if !enabled {
                    if let Some(review) = self.review_state.as_mut() {
                        review.set_free_enabled(false);
                    }
                }
                vec![ReviewEffect::SettingsChanged]
            }
            ReviewAction::ToggleNotesSetting => {
                let enabled = !self.settings().show_notes_button();
                self.settings.set_show_notes_button(enabled);
                vec![ReviewEffect::SettingsChanged]
            }
            ReviewAction::ToggleLockSetting => {
                let enabled = !self.settings().show_lock_button();
                self.settings.set_show_lock_button(enabled);
                vec![ReviewEffect::SettingsChanged]
            }
            ReviewAction::ToggleFlipSetting => {
                let enabled = !self.settings().show_flip_button();
                self.settings.set_show_flip_button(enabled);
                vec![ReviewEffect::SettingsChanged]
            }
            ReviewAction::ToggleBoardSizeSetting => {
                let enabled = !self.settings().small_board();
                self.settings.set_small_board(enabled);
                vec![ReviewEffect::SettingsChanged]
            }
            ReviewAction::TapSquare(square) => {
                if let Some(review) = self.review_state.as_mut() {
                    review.handle_square_tap(square);
                }
                Vec::new()
            }
            ReviewAction::ChoosePromotion(choice) => {
                if let Some(review) = self.review_state.as_mut() {
                    review.finish_promotion(choice.piece_kind());
                }
                Vec::new()
            }
            ReviewAction::CancelPromotion => {
                if let Some(review) = self.review_state.as_mut() {
                    review.cancel_promotion();
                }
                Vec::new()
            }
            ReviewAction::AnalysisPreviousPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.analysis_previous_page();
                }
                Vec::new()
            }
            ReviewAction::AnalysisNextPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.analysis_next_page();
                }
                Vec::new()
            }
            ReviewAction::ReviewReset => {
                if let Some(review) = self.review_state.as_mut() {
                    review.reset_scratch();
                }
                Vec::new()
            }
            ReviewAction::CloseReviewGamePicker => {
                if let Some(review) = self.review_state.as_mut() {
                    review.close_game_picker();
                }
                Vec::new()
            }
            ReviewAction::ReviewGamePickerPreviousPage => {
                if let Some(review) = self.review_state.as_mut() {
                    review.game_picker_previous_page();
                }
                Vec::new()
            }
            ReviewAction::ReviewFlip => {
                if self.settings().show_flip_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.flip();
                    }
                }
                Vec::new()
            }
            ReviewAction::ReviewToggleFree => {
                if self.settings().show_free_mode_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.toggle_free();
                    }
                }
                Vec::new()
            }
            ReviewAction::ReviewToggleOrientationLock => {
                if self.settings().show_lock_button() {
                    if let Some(review) = self.review_state.as_mut() {
                        review.toggle_orientation_lock();
                    }
                }
                Vec::new()
            }
            ReviewAction::ReviewPrevious => {
                self.navigate_review(false);
                Vec::new()
            }
            ReviewAction::ReviewNext => {
                self.navigate_review(true);
                Vec::new()
            }
            ReviewAction::SelectAnalysisNode(index) => {
                self.select_review_node(index);
                Vec::new()
            }
            ReviewAction::OpenReviewGamePicker => {
                self.open_review_game_picker();
                Vec::new()
            }
            ReviewAction::ReviewGamePickerNextPage => {
                let total = self.review_picker_entry_count();
                if let Some(review) = self.review_state.as_mut() {
                    review.game_picker_next_page(total);
                }
                Vec::new()
            }
            ReviewAction::SelectReviewGame(index) => self.request_review_game(index),
            ReviewAction::ActivateReviewGame(key, game) => {
                self.activate_review_game(&key, *game);
                Vec::new()
            }
        }
    }
}
