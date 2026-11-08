//! Puzzle-only state, typed actions and effects.
//!
//! The product action surface cannot switch into a review workspace:
//! ```compile_fail
//! use chess_core::puzzle_app::PuzzleAction;
//! let action = PuzzleAction::ToggleWorkspace;
//! ```

use crate::{
    parse_uci_move, AnalysisNodeIndex, Board, BoardIntent, BoardSessionStatus, CollectionEntry,
    Color, PendingPromotion, PhysicalBoardSeat, Progress, PromotionChoice, Puzzle,
    PuzzleCollection, TapResult, UciMove, WorkspaceSettings,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PuzzleAction {
    ToggleBoardConnection,
    SendBoardPosition,
    BoardStatus(BoardSessionStatus),
    BoardPosition(String),
    PauseBoardInput,
    Exit,
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
pub enum PuzzleEffect {
    Board(BoardIntent),
    ExitRequested,
    ProgressChanged,
    SettingsChanged,
    CollectionRequested(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoardInput {
    collection: String,
    puzzle: String,
    mode: BoardMode,
    expected: Board,
    ready: bool,
    correction: bool,
    rotated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuzzleState {
    board_status: BoardSessionStatus,
    board_target: Option<String>,
    board_input: Option<BoardInput>,
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
    settings: WorkspaceSettings,
    physical_board_seat: PhysicalBoardSeat,
    settings_open: bool,
    progress: Progress,
    transient_message: Option<String>,
    collection_entries: Vec<CollectionEntry>,
    collection_picker_open: bool,
    collection_picker_page: usize,
    puzzle_goto_input: Option<String>,
    analysis_browser: Option<AnalysisBrowserState>,
    analysis_preview_board: Option<Board>,
}

impl PuzzleState {
    pub fn new(active_collection: ActiveCollection, progress: Progress) -> Self {
        Self::new_with_settings(
            active_collection,
            progress,
            WorkspaceSettings::default(),
            PhysicalBoardSeat::default(),
        )
    }

    pub fn new_with_settings(
        active_collection: ActiveCollection,
        progress: Progress,
        settings: WorkspaceSettings,
        physical_board_seat: PhysicalBoardSeat,
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
            board_status: BoardSessionStatus::Disconnected,
            board_target: None,
            board_input: None,
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
            physical_board_seat,
            settings_open: false,
            progress,
            transient_message: None,
            collection_entries: Vec::new(),
            collection_picker_open: false,
            collection_picker_page: 0,
            puzzle_goto_input: None,
            analysis_browser: None,
            analysis_preview_board: None,
        }
    }

    pub fn set_physical_board_seat(&mut self, seat: PhysicalBoardSeat) {
        self.physical_board_seat = seat;
        self.board_input = None;
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
        self.analysis_browser
            .as_ref()
            .and_then(AnalysisBrowserState::selected_node)
    }

    pub fn analysis_page(&self) -> usize {
        self.analysis_browser
            .as_ref()
            .map_or(0, AnalysisBrowserState::page)
    }

    pub fn analysis_can_previous_page(&self) -> bool {
        self.analysis_browser
            .as_ref()
            .is_some_and(|browser| browser.page() > 0)
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

    pub fn flipped(&self) -> bool {
        self.flipped
    }

    pub fn orientation_locked(&self) -> bool {
        self.orientation_locked
    }

    pub const fn description_visible(&self) -> bool {
        self.description_visible
    }

    pub const fn settings(&self) -> &WorkspaceSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut WorkspaceSettings {
        &mut self.settings
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

    pub fn board_status(&self) -> &BoardSessionStatus {
        &self.board_status
    }
    pub fn board_target(&self) -> Option<&str> {
        self.board_target.as_deref()
    }
    pub fn can_send_position(&self) -> bool {
        self.board_status.idle()
            && self.pending_promotion().is_none()
            && !self.settings_open
            && !self.collection_picker_open
            && !self.puzzle_goto_open()
    }

    pub const fn physical_board_seat(&self) -> crate::PhysicalBoardSeat {
        self.physical_board_seat
    }

    pub fn board_input_status(&self) -> crate::BoardInputStatus {
        use crate::BoardInputStatus as Input;
        match self.board_input.as_ref() {
            None => Input::Off,
            Some(input) if input.ready => Input::Listening,
            Some(input) if input.correction => Input::MatchKindle,
            Some(_) => Input::WaitingForPosition,
        }
    }

    fn handle_board_position(&mut self, placement: &str) -> Vec<PuzzleEffect> {
        let Some(mut input) = self.board_input.take() else {
            return Vec::new();
        };
        if self.board_status.busy() {
            self.board_input = Some(input);
            return Vec::new();
        }
        if !self.board_status.connected()
            || self.analysis_browser_open()
            || self.collection_picker_open
            || self.puzzle_goto_open()
            || self.pending_promotion.is_some()
            || input.collection != self.active_collection.key
            || input.puzzle != self.active_puzzle().id
            || input.mode != self.mode
            || input.expected.placement() != self.board.placement()
        {
            return Vec::new();
        }
        let physical = (placement.len() <= 71)
            .then(|| Board::from_fen(&format!("{placement} w - - 0 1")))
            .and_then(Result::ok);
        let Some(physical) = physical else {
            self.board_input = Some(input);
            return Vec::new();
        };
        let physical = if input.rotated {
            // Sensor coordinates remain physical; grading stays canonical.
            Board::from_fen(&format!("{} w - - 0 1", physical.placement_rotated(true)))
                .expect("rotating a valid placement preserves FEN syntax")
        } else {
            physical
        };
        if physical.placement() == input.expected.placement() {
            input.ready = true;
            input.correction = false;
            self.board_input = Some(input);
            return Vec::new();
        }
        let movement = input
            .ready
            .then(|| crate::infer_board_move(&input.expected, &physical))
            .flatten();
        let Some(movement) = movement else {
            self.board_input = Some(input);
            return Vec::new();
        };
        if self.mode == BoardMode::Solution && self.feedback == SolutionFeedback::Complete {
            self.board_input = Some(input);
            return Vec::new();
        }
        let before = self.board.clone();
        let original_ply = self.solution_ply;
        self.board.apply_uci_move(movement);
        let changed = self.mode == BoardMode::Solution && self.check_solver_move(before, movement);
        input.expected = self.board.clone();
        input.ready = input.expected.placement() == physical.placement();
        input.correction = !input.ready;
        let mut effects = Self::progress_effect(changed);
        if self.mode == BoardMode::Solution && self.solution_ply == original_ply + 2 {
            // Only an accepted physical solver move can authorize this reply.
            // Freeze input until the reply's exact placement is observed; its
            // motor echoes must never become another graded solver move.
            let target = input.expected.placement_rotated(input.rotated);
            input.ready = false;
            input.correction = false;
            self.board_target = Some(target.clone());
            self.board_status = BoardSessionStatus::Sending;
            effects.push(PuzzleEffect::Board(BoardIntent::Send(target)));
        }
        self.board_input = Some(input);
        effects
    }

    pub fn dispatch(&mut self, action: PuzzleAction) -> Vec<PuzzleEffect> {
        // Board-editing/navigation actions invalidate a physical transaction.
        // Flip and Settings preserve absolute sensor coordinates and listening.
        if matches!(
            action,
            PuzzleAction::PauseBoardInput
                | PuzzleAction::TapSquare(_)
                | PuzzleAction::ChoosePromotion(_)
                | PuzzleAction::CancelPromotion
                | PuzzleAction::PreviousPuzzle
                | PuzzleAction::NextPuzzle
                | PuzzleAction::Reset
                | PuzzleAction::ToggleMode
                | PuzzleAction::ToggleAnalysis
                | PuzzleAction::OpenAnalysis
                | PuzzleAction::CloseAnalysis
                | PuzzleAction::SelectAnalysisNode(_)
                | PuzzleAction::OpenCollectionPicker
                | PuzzleAction::ActivateCollection(_)
                | PuzzleAction::OpenPuzzleGoto
                | PuzzleAction::ConfirmPuzzleGoto
        ) {
            self.board_input = None;
        }
        // Session updates pass through ordinary modals. A sent target is an
        // immutable snapshot independent of subsequent board navigation.
        let intent = match &action {
            PuzzleAction::CyclePhysicalBoardSeat => {
                if !self.settings_open {
                    return Vec::new();
                }
                let seat = self.physical_board_seat().next();
                self.physical_board_seat = seat;
                self.board_input = None;
                return vec![PuzzleEffect::SettingsChanged];
            }
            PuzzleAction::BoardPosition(placement) => return self.handle_board_position(placement),
            PuzzleAction::PauseBoardInput => return Vec::new(),
            PuzzleAction::BoardStatus(status) => {
                if !status.connected() {
                    self.board_input = None;
                }
                if status.busy() && self.board_target.is_none() {
                    return Vec::new();
                }
                self.board_status = status.clone();
                if !status.busy() {
                    self.board_target = None;
                }
                return Vec::new();
            }
            PuzzleAction::ToggleBoardConnection => {
                if self.board_status.connected() {
                    self.board_status = BoardSessionStatus::Disconnected;
                    self.board_target = None;
                    self.board_input = None;
                    Some(BoardIntent::Disconnect)
                } else if matches!(
                    self.board_status,
                    BoardSessionStatus::Disconnected | BoardSessionStatus::Error(_)
                ) {
                    self.board_status = BoardSessionStatus::Connecting;
                    Some(BoardIntent::Connect)
                } else {
                    None
                }
            }
            PuzzleAction::SendBoardPosition if self.can_send_position() && !self.settings_open => {
                let rotated = self
                    .physical_board_seat()
                    .rotates(self.active_puzzle().side_to_move());
                let target = self.board().placement_rotated(rotated);
                self.board_input = (!self.analysis_browser_open()).then(|| BoardInput {
                    collection: self.active_collection.key.clone(),
                    puzzle: self.active_puzzle().id.clone(),
                    mode: self.mode,
                    expected: self.board().clone(),
                    ready: false,
                    correction: false,
                    rotated,
                });
                self.board_target = Some(target.clone());
                self.board_status = BoardSessionStatus::Sending;
                Some(BoardIntent::Send(target))
            }
            PuzzleAction::SendBoardPosition => return Vec::new(),
            _ => None,
        };
        if let Some(intent) = intent {
            return vec![PuzzleEffect::Board(intent)];
        }

        if self.settings_open
            && !matches!(
                &action,
                PuzzleAction::Exit
                    | PuzzleAction::CloseSettings
                    | PuzzleAction::ToggleFreeModeSetting
                    | PuzzleAction::ToggleNotesSetting
                    | PuzzleAction::ToggleLockSetting
                    | PuzzleAction::ToggleFlipSetting
                    | PuzzleAction::ToggleBoardSizeSetting
                    | PuzzleAction::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        if self.puzzle_goto_input.is_some()
            && !matches!(
                &action,
                PuzzleAction::Exit
                    | PuzzleAction::PuzzleGotoDigit(_)
                    | PuzzleAction::PuzzleGotoBackspace
                    | PuzzleAction::ConfirmPuzzleGoto
                    | PuzzleAction::CancelPuzzleGoto
                    | PuzzleAction::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        if self.collection_picker_open
            && !matches!(
                &action,
                PuzzleAction::Exit
                    | PuzzleAction::CloseCollectionPicker
                    | PuzzleAction::CollectionPickerPreviousPage
                    | PuzzleAction::CollectionPickerNextPage
                    | PuzzleAction::SelectCollection(_)
                    | PuzzleAction::ActivateCollection(_)
                    | PuzzleAction::SetTransientMessage(_)
            )
        {
            return Vec::new();
        }

        if self.analysis_browser.is_some()
            && matches!(
                &action,
                PuzzleAction::TapSquare(_)
                    | PuzzleAction::ChoosePromotion(_)
                    | PuzzleAction::CancelPromotion
            )
        {
            return Vec::new();
        }

        match action {
            PuzzleAction::ToggleBoardConnection
            | PuzzleAction::SendBoardPosition
            | PuzzleAction::BoardStatus(_)
            | PuzzleAction::BoardPosition(_)
            | PuzzleAction::PauseBoardInput
            | PuzzleAction::CyclePhysicalBoardSeat => Vec::new(),
            PuzzleAction::Exit => vec![PuzzleEffect::ExitRequested],
            PuzzleAction::OpenSettings => {
                self.open_settings();
                Vec::new()
            }
            PuzzleAction::CloseSettings => {
                self.settings_open = false;
                Vec::new()
            }
            PuzzleAction::ToggleFreeModeSetting => {
                let enabled = !self.settings.show_free_mode_button();
                self.settings.set_show_free_mode_button(enabled);
                if !enabled && self.mode == BoardMode::FreeBoard {
                    self.toggle_mode();
                }
                vec![PuzzleEffect::SettingsChanged]
            }
            PuzzleAction::ToggleNotesSetting => {
                let enabled = !self.settings.show_notes_button();
                self.settings.set_show_notes_button(enabled);
                if !enabled {
                    self.description_visible = false;
                }
                vec![PuzzleEffect::SettingsChanged]
            }
            PuzzleAction::ToggleLockSetting => {
                let enabled = !self.settings().show_lock_button();
                self.settings_mut().set_show_lock_button(enabled);
                vec![PuzzleEffect::SettingsChanged]
            }
            PuzzleAction::ToggleFlipSetting => {
                let enabled = !self.settings().show_flip_button();
                self.settings_mut().set_show_flip_button(enabled);
                vec![PuzzleEffect::SettingsChanged]
            }
            PuzzleAction::ToggleBoardSizeSetting => {
                let small = !self.settings().small_board();
                self.settings_mut().set_small_board(small);
                vec![PuzzleEffect::SettingsChanged]
            }
            PuzzleAction::OpenCollectionPicker => {
                self.open_collection_picker();
                Vec::new()
            }
            PuzzleAction::CloseCollectionPicker => {
                self.collection_picker_open = false;
                Vec::new()
            }
            PuzzleAction::CollectionPickerPreviousPage => {
                self.collection_picker_page = self.collection_picker_page.saturating_sub(1);
                Vec::new()
            }
            PuzzleAction::CollectionPickerNextPage => {
                if self.collection_picker_can_next_page() {
                    self.collection_picker_page += 1;
                }
                Vec::new()
            }
            PuzzleAction::SelectCollection(index) => self.request_collection(index),
            PuzzleAction::TapSquare(square) => {
                Self::progress_effect(self.handle_square_tap(square))
            }
            PuzzleAction::ChoosePromotion(choice) => {
                Self::progress_effect(self.finish_promotion(choice))
            }
            PuzzleAction::CancelPromotion => {
                self.cancel_promotion();
                Vec::new()
            }
            PuzzleAction::ToggleAnalysis => {
                if self.analysis_browser_open() {
                    self.close_analysis();
                } else {
                    self.open_analysis();
                }
                Vec::new()
            }
            PuzzleAction::OpenAnalysis => {
                self.open_analysis();
                Vec::new()
            }
            PuzzleAction::CloseAnalysis => {
                self.close_analysis();
                Vec::new()
            }
            PuzzleAction::SelectAnalysisNode(index) => {
                self.select_analysis_node(index);
                Vec::new()
            }
            PuzzleAction::AnalysisPreviousPage => {
                self.analysis_previous_page();
                Vec::new()
            }
            PuzzleAction::AnalysisNextPage => {
                self.analysis_next_page();
                Vec::new()
            }
            PuzzleAction::OpenPuzzleGoto => {
                self.open_puzzle_goto();
                Vec::new()
            }
            PuzzleAction::PuzzleGotoDigit(digit) => {
                self.append_puzzle_goto_digit(digit);
                Vec::new()
            }
            PuzzleAction::PuzzleGotoBackspace => {
                self.puzzle_goto_backspace();
                Vec::new()
            }
            PuzzleAction::ConfirmPuzzleGoto => Self::progress_effect(self.confirm_puzzle_goto()),
            PuzzleAction::CancelPuzzleGoto => {
                self.puzzle_goto_input = None;
                Vec::new()
            }
            PuzzleAction::PreviousPuzzle => Self::progress_effect(self.turn_puzzle(false)),
            PuzzleAction::NextPuzzle => Self::progress_effect(self.turn_puzzle(true)),
            PuzzleAction::Reset => {
                self.board.reset();
                self.reset_attempt();
                self.transient_message = None;
                Vec::new()
            }
            PuzzleAction::Flip => {
                if self.settings().show_flip_button() {
                    self.flipped = !self.flipped;
                }
                Vec::new()
            }
            PuzzleAction::ToggleMode => {
                if self.settings().show_free_mode_button() {
                    self.toggle_mode();
                }
                Vec::new()
            }
            PuzzleAction::ToggleOrientationLock => {
                if self.settings().show_lock_button() {
                    self.orientation_locked = !self.orientation_locked;
                }
                Vec::new()
            }
            PuzzleAction::ToggleDescription => {
                if self.settings.show_notes_button() {
                    self.description_visible = !self.description_visible;
                }
                Vec::new()
            }
            PuzzleAction::ActivateCollection(collection) => {
                let changed = self.activate_collection(collection);
                self.collection_picker_open = false;
                Self::progress_effect(changed)
            }
            PuzzleAction::SetTransientMessage(message) => {
                self.transient_message = message;
                Vec::new()
            }
        }
    }

    fn progress_effect(changed: bool) -> Vec<PuzzleEffect> {
        if changed {
            vec![PuzzleEffect::ProgressChanged]
        } else {
            Vec::new()
        }
    }

    fn open_settings(&mut self) {
        if self.settings_open
            || self.pending_promotion().is_some()
            || self.collection_picker_open
            || self.puzzle_goto_input.is_some()
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

    fn request_collection(&self, index: usize) -> Vec<PuzzleEffect> {
        if !self.collection_picker_open || !self.collection_picker_visible_range().contains(&index)
        {
            return Vec::new();
        }

        self.collection_entries
            .get(index)
            .map(|entry| vec![PuzzleEffect::CollectionRequested(entry.key().to_owned())])
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
        self.board_input = None;
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
