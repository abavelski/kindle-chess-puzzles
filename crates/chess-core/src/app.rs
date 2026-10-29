//! Pure application state for puzzle solving and game-review workspaces.

use crate::{
    parse_uci_move, AnalysisNodeIndex, Board, CollectionEntry, Color, PieceKind, Progress, Puzzle,
    PuzzleCollection, ReviewFileError, ReviewGame, ReviewGameEntry, ReviewGameKey, ReviewState,
    Settings, TapResult, UciMove, Workspace, WorkspaceSettings,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnalysisBrowserState {
    selected_node: Option<AnalysisNodeIndex>,
    page: usize,
}

impl AnalysisBrowserState {
    pub const fn selected_node(&self) -> Option<AnalysisNodeIndex> {
        self.selected_node
    }

    pub const fn page(&self) -> usize {
        self.page
    }
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
    pub(crate) before: Board,
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) color: Color,
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
    ExitRequested,
    ProgressChanged,
    SettingsChanged,
    CollectionRequested(String),
    ReviewGameRequested(ReviewGameKey),
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
    settings: Settings,
    settings_open: bool,
    progress: Progress,
    transient_message: Option<String>,
    collection_entries: Vec<CollectionEntry>,
    collection_picker_open: bool,
    collection_picker_page: usize,
    puzzle_goto_input: Option<String>,
    analysis_browser: Option<AnalysisBrowserState>,
    analysis_preview_board: Option<Board>,
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
            settings,
            settings_open: false,
            progress,
            transient_message: None,
            collection_entries: Vec::new(),
            collection_picker_open: false,
            collection_picker_page: 0,
            puzzle_goto_input: None,
            analysis_browser: None,
            analysis_preview_board: None,
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
                self.transient_message = Some("No review games are available.".to_owned());
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

    /// Restore only review state. Missing content never changes puzzle state or progress.
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

    pub const fn puzzle_goto_open(&self) -> bool {
        self.puzzle_goto_input.is_some()
    }

    pub fn puzzle_goto_input(&self) -> Option<&str> {
        self.puzzle_goto_input.as_deref()
    }

    pub fn puzzle_goto_number(&self) -> Option<usize> {
        self.puzzle_goto_input
            .as_deref()
            .filter(|input| !input.is_empty())
            .and_then(|input| input.parse::<usize>().ok())
            .filter(|number| *number > 0 && *number <= self.active_collection.puzzles.len())
    }

    pub fn active_puzzle(&self) -> &Puzzle {
        &self.active_collection.puzzles[self.puzzle_index]
    }

    pub fn board(&self) -> &Board {
        if self.workspace == Workspace::Review {
            if let Some(review) = self.review_state.as_ref() {
                return review.board();
            }
        }
        self.analysis_preview_board.as_ref().unwrap_or(&self.board)
    }

    pub const fn live_board(&self) -> &Board {
        &self.board
    }

    pub fn analysis_available(&self) -> bool {
        self.active_puzzle().analysis.is_some()
    }

    pub const fn analysis_browser_open(&self) -> bool {
        self.analysis_browser.is_some()
    }

    pub fn analysis_browser(&self) -> Option<&AnalysisBrowserState> {
        self.analysis_browser.as_ref()
    }

    pub fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        match self.workspace {
            Workspace::Puzzles => self
                .analysis_browser
                .as_ref()
                .and_then(|browser| browser.selected_node()),
            Workspace::Review => self.review_state.as_ref().map(ReviewState::selected_node),
        }
    }

    pub fn analysis_page(&self) -> usize {
        match self.workspace {
            Workspace::Puzzles => self
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
        self.mode
    }

    pub const fn solution_ply(&self) -> usize {
        self.solution_ply
    }

    pub const fn feedback(&self) -> SolutionFeedback {
        self.feedback
    }

    pub fn pending_promotion(&self) -> Option<&PendingPromotion> {
        match self.workspace {
            Workspace::Puzzles => self.pending_promotion.as_ref(),
            Workspace::Review => self
                .review_state
                .as_ref()
                .and_then(ReviewState::pending_promotion),
        }
    }

    pub fn flipped(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.flipped,
            Workspace::Review => self.review_state.as_ref().is_some_and(ReviewState::flipped),
        }
    }

    pub fn orientation_locked(&self) -> bool {
        match self.workspace {
            Workspace::Puzzles => self.orientation_locked,
            Workspace::Review => self
                .review_state
                .as_ref()
                .is_some_and(ReviewState::orientation_locked),
        }
    }

    pub const fn description_visible(&self) -> bool {
        self.description_visible
    }

    pub const fn settings(&self) -> &WorkspaceSettings {
        self.settings.for_workspace(self.workspace)
    }

    /// Both workspace profiles for durable storage, regardless of the active view.
    pub const fn persisted_settings(&self) -> &Settings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut WorkspaceSettings {
        self.settings.for_workspace_mut(self.workspace)
    }

    pub const fn settings_open(&self) -> bool {
        self.settings_open
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
        if self.settings_open
            && !matches!(
                &action,
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

        if self.puzzle_goto_input.is_some()
            && !matches!(
                &action,
                Action::Exit
                    | Action::PuzzleGotoDigit(_)
                    | Action::PuzzleGotoBackspace
                    | Action::ConfirmPuzzleGoto
                    | Action::CancelPuzzleGoto
                    | Action::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        if self.collection_picker_open
            && !matches!(
                &action,
                Action::Exit
                    | Action::CloseCollectionPicker
                    | Action::CollectionPickerPreviousPage
                    | Action::CollectionPickerNextPage
                    | Action::SelectCollection(_)
                    | Action::ActivateCollection(_)
                    | Action::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        if self
            .review_state
            .as_ref()
            .is_some_and(ReviewState::game_picker_open)
            && !matches!(
                &action,
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

        if self.workspace == Workspace::Puzzles
            && self.analysis_browser.is_some()
            && matches!(
                &action,
                Action::TapSquare(_) | Action::ChoosePromotion(_) | Action::CancelPromotion
            )
        {
            return Vec::new();
        }

        if self.workspace == Workspace::Puzzles
            && matches!(
                &action,
                Action::ReviewPrevious
                    | Action::ReviewNext
                    | Action::ReviewReset
                    | Action::ReviewFlip
                    | Action::ReviewToggleFree
                    | Action::ReviewToggleOrientationLock
                    | Action::OpenReviewGamePicker
                    | Action::CloseReviewGamePicker
                    | Action::ReviewGamePickerPreviousPage
                    | Action::ReviewGamePickerNextPage
                    | Action::SelectReviewGame(_)
                    | Action::ActivateReviewGame(_, _)
            )
        {
            return Vec::new();
        }

        if self.workspace == Workspace::Review
            && !matches!(
                &action,
                Action::Exit
                    | Action::ToggleWorkspace
                    | Action::TapSquare(_)
                    | Action::ChoosePromotion(_)
                    | Action::CancelPromotion
                    | Action::SelectAnalysisNode(_)
                    | Action::AnalysisPreviousPage
                    | Action::AnalysisNextPage
                    | Action::ReviewPrevious
                    | Action::ReviewNext
                    | Action::ReviewReset
                    | Action::ReviewFlip
                    | Action::ReviewToggleFree
                    | Action::ReviewToggleOrientationLock
                    | Action::OpenReviewGamePicker
                    | Action::CloseReviewGamePicker
                    | Action::ReviewGamePickerPreviousPage
                    | Action::ReviewGamePickerNextPage
                    | Action::SelectReviewGame(_)
                    | Action::ActivateReviewGame(_, _)
                    | Action::OpenSettings
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

        match action {
            Action::Exit => vec![Effect::ExitRequested],
            Action::ToggleWorkspace => {
                self.toggle_workspace();
                Vec::new()
            }
            Action::OpenSettings => {
                self.open_settings();
                Vec::new()
            }
            Action::CloseSettings => {
                self.settings_open = false;
                Vec::new()
            }
            Action::ToggleFreeModeSetting => {
                let enabled = !self.settings().show_free_mode_button();
                self.settings_mut().set_show_free_mode_button(enabled);
                if !enabled {
                    if self.workspace == Workspace::Puzzles && self.mode == BoardMode::FreeBoard {
                        self.toggle_mode();
                    }
                    if self.workspace == Workspace::Review {
                        if let Some(review) = self.review_state.as_mut() {
                            review.set_free_enabled(false);
                        }
                    }
                }
                vec![Effect::SettingsChanged]
            }
            Action::ToggleNotesSetting => {
                let enabled = !self.settings().show_notes_button();
                self.settings_mut().set_show_notes_button(enabled);
                if !enabled && self.workspace == Workspace::Puzzles {
                    self.description_visible = false;
                }
                vec![Effect::SettingsChanged]
            }
            Action::ToggleLockSetting => {
                let enabled = !self.settings().show_lock_button();
                self.settings_mut().set_show_lock_button(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::ToggleFlipSetting => {
                let enabled = !self.settings().show_flip_button();
                self.settings_mut().set_show_flip_button(enabled);
                vec![Effect::SettingsChanged]
            }
            Action::ToggleBoardSizeSetting => {
                let small = !self.settings().small_board();
                self.settings_mut().set_small_board(small);
                vec![Effect::SettingsChanged]
            }
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
            Action::TapSquare(square) => match self.workspace {
                Workspace::Puzzles => Self::progress_effect(self.handle_square_tap(square)),
                Workspace::Review => {
                    if let Some(review) = self.review_state.as_mut() {
                        review.handle_square_tap(square);
                    }
                    Vec::new()
                }
            },
            Action::ChoosePromotion(choice) => match self.workspace {
                Workspace::Puzzles => Self::progress_effect(self.finish_promotion(choice)),
                Workspace::Review => {
                    if let Some(review) = self.review_state.as_mut() {
                        review.finish_promotion(choice.piece_kind());
                    }
                    Vec::new()
                }
            },
            Action::CancelPromotion => {
                match self.workspace {
                    Workspace::Puzzles => self.cancel_promotion(),
                    Workspace::Review => {
                        if let Some(review) = self.review_state.as_mut() {
                            review.cancel_promotion();
                        }
                    }
                }
                Vec::new()
            }
            Action::ToggleAnalysis => {
                if self.analysis_browser_open() {
                    self.close_analysis();
                } else {
                    self.open_analysis();
                }
                Vec::new()
            }
            Action::OpenAnalysis => {
                self.open_analysis();
                Vec::new()
            }
            Action::CloseAnalysis => {
                self.close_analysis();
                Vec::new()
            }
            Action::SelectAnalysisNode(index) => {
                match self.workspace {
                    Workspace::Puzzles => self.select_analysis_node(index),
                    Workspace::Review => self.select_review_node(index),
                }
                Vec::new()
            }
            Action::AnalysisPreviousPage => {
                match self.workspace {
                    Workspace::Puzzles => self.analysis_previous_page(),
                    Workspace::Review => {
                        if let Some(review) = self.review_state.as_mut() {
                            review.analysis_previous_page();
                        }
                    }
                }
                Vec::new()
            }
            Action::AnalysisNextPage => {
                match self.workspace {
                    Workspace::Puzzles => self.analysis_next_page(),
                    Workspace::Review => {
                        if let Some(review) = self.review_state.as_mut() {
                            review.analysis_next_page();
                        }
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
            Action::ReviewReset => {
                if let Some(review) = self.review_state.as_mut() {
                    review.reset_scratch();
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
            Action::OpenReviewGamePicker => {
                self.open_review_game_picker();
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
            Action::OpenPuzzleGoto => {
                self.open_puzzle_goto();
                Vec::new()
            }
            Action::PuzzleGotoDigit(digit) => {
                self.append_puzzle_goto_digit(digit);
                Vec::new()
            }
            Action::PuzzleGotoBackspace => {
                self.puzzle_goto_backspace();
                Vec::new()
            }
            Action::ConfirmPuzzleGoto => Self::progress_effect(self.confirm_puzzle_goto()),
            Action::CancelPuzzleGoto => {
                self.puzzle_goto_input = None;
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
                if self.settings().show_flip_button() {
                    self.flipped = !self.flipped;
                }
                Vec::new()
            }
            Action::ToggleMode => {
                if self.settings().show_free_mode_button() {
                    self.toggle_mode();
                }
                Vec::new()
            }
            Action::ToggleOrientationLock => {
                if self.settings().show_lock_button() {
                    self.orientation_locked = !self.orientation_locked;
                }
                Vec::new()
            }
            Action::ToggleDescription => {
                if self.settings.show_notes_button() {
                    self.description_visible = !self.description_visible;
                }
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

    fn toggle_workspace(&mut self) {
        match self.workspace {
            Workspace::Puzzles => {
                if self.review_state.is_some() {
                    self.workspace = Workspace::Review;
                    self.transient_message = None;
                } else {
                    self.transient_message = Some("No review games are available.".to_owned());
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

    fn progress_effect(changed: bool) -> Vec<Effect> {
        if changed {
            vec![Effect::ProgressChanged]
        } else {
            Vec::new()
        }
    }

    fn open_settings(&mut self) {
        if self.settings_open
            || self.pending_promotion().is_some()
            || self.collection_picker_open
            || self.puzzle_goto_input.is_some()
            || self
                .review_state
                .as_ref()
                .is_some_and(ReviewState::game_picker_open)
        {
            return;
        }
        self.settings_open = true;
    }

    fn open_analysis(&mut self) {
        if self.analysis_browser.is_some()
            || self.pending_promotion.is_some()
            || !self.analysis_available()
        {
            return;
        }

        self.analysis_browser = Some(AnalysisBrowserState::default());
        self.analysis_preview_board = None;
    }

    fn close_analysis(&mut self) {
        self.analysis_browser = None;
        self.analysis_preview_board = None;
    }

    fn select_analysis_node(&mut self, index: AnalysisNodeIndex) {
        if self.analysis_browser.is_none() {
            return;
        }

        let preview_board = self
            .active_puzzle()
            .analysis
            .as_ref()
            .and_then(|analysis| analysis.node(index))
            .map(|node| Board::from_fen(&node.fen).expect("parsed analysis FEN remains valid"));
        let Some(preview_board) = preview_board else {
            return;
        };

        if let Some(browser) = self.analysis_browser.as_mut() {
            browser.selected_node = Some(index);
        }
        self.analysis_preview_board = Some(preview_board);
    }

    fn analysis_previous_page(&mut self) {
        if let Some(browser) = self.analysis_browser.as_mut() {
            browser.page = browser.page.saturating_sub(1);
        }
    }

    fn analysis_next_page(&mut self) {
        if let Some(browser) = self.analysis_browser.as_mut() {
            browser.page = browser.page.saturating_add(1);
        }
    }

    fn open_puzzle_goto(&mut self) {
        if self.puzzle_goto_input.is_some()
            || self.pending_promotion.is_some()
            || self.collection_picker_open
            || self.settings_open
        {
            return;
        }
        self.puzzle_goto_input = Some(String::new());
    }

    fn append_puzzle_goto_digit(&mut self, digit: u8) {
        if digit > 9 {
            return;
        }
        let max_puzzle = self.active_collection.puzzles.len();
        let Some(input) = self.puzzle_goto_input.as_mut() else {
            return;
        };
        if input.is_empty() && digit == 0 {
            return;
        }

        let mut candidate = input.clone();
        candidate.push(char::from(b'0' + digit));
        if candidate
            .parse::<usize>()
            .is_ok_and(|number| number > 0 && number <= max_puzzle)
        {
            *input = candidate;
        }
    }

    fn puzzle_goto_backspace(&mut self) {
        if let Some(input) = self.puzzle_goto_input.as_mut() {
            input.pop();
        }
    }

    fn confirm_puzzle_goto(&mut self) -> bool {
        let Some(number) = self.puzzle_goto_number() else {
            return false;
        };
        let index = number - 1;
        self.puzzle_goto_input = None;
        if index == self.puzzle_index {
            return false;
        }

        self.select_puzzle(index);
        self.remember_current_puzzle()
    }

    fn open_collection_picker(&mut self) {
        if self.collection_entries.is_empty()
            || self.pending_promotion.is_some()
            || self.settings_open
        {
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
            self.description_visible = self.settings.show_notes_button();
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
        self.close_analysis();
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
        self.close_analysis();
        self.description_visible = false;
        self.solution_ply = 0;
        self.feedback = SolutionFeedback::None;
        self.pending_promotion = None;
        self.puzzle_goto_input = None;
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
