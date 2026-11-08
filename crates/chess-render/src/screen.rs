//! Read-only product screen inputs; shared drawing lives in eink-ui.
use chess_core::*;
use chess_core::{puzzle_app::PuzzleState, review_app::ReviewApp};

pub trait ScreenState {
    fn workspace_toggle_visible(&self) -> bool {
        false
    }
    fn active_collection(&self) -> &ActiveCollection {
        unreachable!("product-specific screen accessor")
    }
    fn active_puzzle(&self) -> &Puzzle {
        unreachable!("product-specific screen accessor")
    }
    fn active_puzzle_index(&self) -> usize {
        0
    }
    fn active_review_game(&self) -> Option<&ReviewGameEntry> {
        None
    }
    fn analysis_available(&self) -> bool {
        false
    }
    fn analysis_browser_open(&self) -> bool {
        false
    }
    fn analysis_page(&self) -> usize {
        0
    }
    fn board(&self) -> &Board {
        unreachable!("product-specific screen accessor")
    }
    fn board_status(&self) -> &BoardSessionStatus {
        &BoardSessionStatus::Disconnected
    }
    fn can_next_puzzle(&self) -> bool {
        false
    }
    fn can_previous_puzzle(&self) -> bool {
        false
    }
    fn can_send_position(&self) -> bool {
        false
    }
    fn collection_entries(&self) -> &[CollectionEntry] {
        &[]
    }
    fn collection_picker_can_next_page(&self) -> bool {
        false
    }
    fn collection_picker_can_previous_page(&self) -> bool {
        false
    }
    fn collection_picker_open(&self) -> bool {
        false
    }
    fn collection_picker_page(&self) -> usize {
        0
    }
    fn collection_picker_page_count(&self) -> usize {
        0
    }
    fn collection_picker_visible_range(&self) -> std::ops::Range<usize> {
        0..0
    }
    fn description_visible(&self) -> bool {
        false
    }
    fn feedback(&self) -> SolutionFeedback {
        SolutionFeedback::None
    }
    fn flipped(&self) -> bool {
        false
    }
    fn is_current_solved(&self) -> bool {
        false
    }
    fn mode(&self) -> BoardMode {
        BoardMode::Solution
    }
    fn orientation_locked(&self) -> bool {
        false
    }
    fn pending_promotion(&self) -> Option<&PendingPromotion> {
        None
    }
    fn physical_board_seat(&self) -> PhysicalBoardSeat {
        PhysicalBoardSeat::Changing
    }
    fn puzzle_goto_input(&self) -> Option<&str> {
        None
    }
    fn puzzle_goto_number(&self) -> Option<usize> {
        None
    }
    fn puzzle_goto_open(&self) -> bool {
        false
    }
    fn review_picker_entry_count(&self) -> usize {
        0
    }
    fn review_picker_error(&self, _picker_index: usize) -> Option<&ReviewFileError> {
        None
    }
    fn review_picker_game(&self, _picker_index: usize) -> Option<&ReviewGameEntry> {
        None
    }
    fn review_state(&self) -> Option<&ReviewState> {
        None
    }
    fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        None
    }
    fn settings(&self) -> &WorkspaceSettings {
        unreachable!("product-specific screen accessor")
    }
    fn settings_open(&self) -> bool {
        false
    }
    fn transient_message(&self) -> Option<&str> {
        None
    }
    fn workspace(&self) -> Workspace {
        Workspace::Puzzles
    }
}
impl ScreenState for AppState {
    fn workspace_toggle_visible(&self) -> bool {
        true
    }
    fn active_collection(&self) -> &ActiveCollection {
        AppState::active_collection(self)
    }
    fn active_puzzle(&self) -> &Puzzle {
        AppState::active_puzzle(self)
    }
    fn active_puzzle_index(&self) -> usize {
        AppState::active_puzzle_index(self)
    }
    fn active_review_game(&self) -> Option<&ReviewGameEntry> {
        AppState::active_review_game(self)
    }
    fn analysis_available(&self) -> bool {
        AppState::analysis_available(self)
    }
    fn analysis_browser_open(&self) -> bool {
        AppState::analysis_browser_open(self)
    }
    fn analysis_page(&self) -> usize {
        AppState::analysis_page(self)
    }
    fn board(&self) -> &Board {
        AppState::board(self)
    }
    fn board_status(&self) -> &BoardSessionStatus {
        AppState::board_status(self)
    }
    fn can_next_puzzle(&self) -> bool {
        AppState::can_next_puzzle(self)
    }
    fn can_previous_puzzle(&self) -> bool {
        AppState::can_previous_puzzle(self)
    }
    fn can_send_position(&self) -> bool {
        AppState::can_send_position(self)
    }
    fn collection_entries(&self) -> &[CollectionEntry] {
        AppState::collection_entries(self)
    }
    fn collection_picker_can_next_page(&self) -> bool {
        AppState::collection_picker_can_next_page(self)
    }
    fn collection_picker_can_previous_page(&self) -> bool {
        AppState::collection_picker_can_previous_page(self)
    }
    fn collection_picker_open(&self) -> bool {
        AppState::collection_picker_open(self)
    }
    fn collection_picker_page(&self) -> usize {
        AppState::collection_picker_page(self)
    }
    fn collection_picker_page_count(&self) -> usize {
        AppState::collection_picker_page_count(self)
    }
    fn collection_picker_visible_range(&self) -> std::ops::Range<usize> {
        AppState::collection_picker_visible_range(self)
    }
    fn description_visible(&self) -> bool {
        AppState::description_visible(self)
    }
    fn feedback(&self) -> SolutionFeedback {
        AppState::feedback(self)
    }
    fn flipped(&self) -> bool {
        AppState::flipped(self)
    }
    fn is_current_solved(&self) -> bool {
        AppState::is_current_solved(self)
    }
    fn mode(&self) -> BoardMode {
        AppState::mode(self)
    }
    fn orientation_locked(&self) -> bool {
        AppState::orientation_locked(self)
    }
    fn pending_promotion(&self) -> Option<&PendingPromotion> {
        AppState::pending_promotion(self)
    }
    fn physical_board_seat(&self) -> PhysicalBoardSeat {
        AppState::physical_board_seat(self)
    }
    fn puzzle_goto_input(&self) -> Option<&str> {
        AppState::puzzle_goto_input(self)
    }
    fn puzzle_goto_number(&self) -> Option<usize> {
        AppState::puzzle_goto_number(self)
    }
    fn puzzle_goto_open(&self) -> bool {
        AppState::puzzle_goto_open(self)
    }
    fn review_picker_entry_count(&self) -> usize {
        AppState::review_picker_entry_count(self)
    }
    fn review_picker_error(&self, picker_index: usize) -> Option<&ReviewFileError> {
        AppState::review_picker_error(self, picker_index)
    }
    fn review_picker_game(&self, picker_index: usize) -> Option<&ReviewGameEntry> {
        AppState::review_picker_game(self, picker_index)
    }
    fn review_state(&self) -> Option<&ReviewState> {
        AppState::review_state(self)
    }
    fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        AppState::selected_analysis_node(self)
    }
    fn settings(&self) -> &WorkspaceSettings {
        AppState::settings(self)
    }
    fn settings_open(&self) -> bool {
        AppState::settings_open(self)
    }
    fn transient_message(&self) -> Option<&str> {
        AppState::transient_message(self)
    }
    fn workspace(&self) -> Workspace {
        AppState::workspace(self)
    }
}
impl ScreenState for PuzzleState {
    fn active_collection(&self) -> &ActiveCollection {
        PuzzleState::active_collection(self)
    }
    fn active_puzzle(&self) -> &Puzzle {
        PuzzleState::active_puzzle(self)
    }
    fn active_puzzle_index(&self) -> usize {
        PuzzleState::active_puzzle_index(self)
    }
    fn analysis_available(&self) -> bool {
        PuzzleState::analysis_available(self)
    }
    fn analysis_browser_open(&self) -> bool {
        PuzzleState::analysis_browser_open(self)
    }
    fn analysis_page(&self) -> usize {
        PuzzleState::analysis_page(self)
    }
    fn board(&self) -> &Board {
        PuzzleState::board(self)
    }
    fn board_status(&self) -> &BoardSessionStatus {
        PuzzleState::board_status(self)
    }
    fn can_next_puzzle(&self) -> bool {
        PuzzleState::can_next_puzzle(self)
    }
    fn can_previous_puzzle(&self) -> bool {
        PuzzleState::can_previous_puzzle(self)
    }
    fn can_send_position(&self) -> bool {
        PuzzleState::can_send_position(self)
    }
    fn collection_entries(&self) -> &[CollectionEntry] {
        PuzzleState::collection_entries(self)
    }
    fn collection_picker_can_next_page(&self) -> bool {
        PuzzleState::collection_picker_can_next_page(self)
    }
    fn collection_picker_can_previous_page(&self) -> bool {
        PuzzleState::collection_picker_can_previous_page(self)
    }
    fn collection_picker_open(&self) -> bool {
        PuzzleState::collection_picker_open(self)
    }
    fn collection_picker_page(&self) -> usize {
        PuzzleState::collection_picker_page(self)
    }
    fn collection_picker_page_count(&self) -> usize {
        PuzzleState::collection_picker_page_count(self)
    }
    fn collection_picker_visible_range(&self) -> std::ops::Range<usize> {
        PuzzleState::collection_picker_visible_range(self)
    }
    fn description_visible(&self) -> bool {
        PuzzleState::description_visible(self)
    }
    fn feedback(&self) -> SolutionFeedback {
        PuzzleState::feedback(self)
    }
    fn flipped(&self) -> bool {
        PuzzleState::flipped(self)
    }
    fn is_current_solved(&self) -> bool {
        PuzzleState::is_current_solved(self)
    }
    fn mode(&self) -> BoardMode {
        PuzzleState::mode(self)
    }
    fn orientation_locked(&self) -> bool {
        PuzzleState::orientation_locked(self)
    }
    fn pending_promotion(&self) -> Option<&PendingPromotion> {
        PuzzleState::pending_promotion(self)
    }
    fn physical_board_seat(&self) -> PhysicalBoardSeat {
        PuzzleState::physical_board_seat(self)
    }
    fn puzzle_goto_input(&self) -> Option<&str> {
        PuzzleState::puzzle_goto_input(self)
    }
    fn puzzle_goto_number(&self) -> Option<usize> {
        PuzzleState::puzzle_goto_number(self)
    }
    fn puzzle_goto_open(&self) -> bool {
        PuzzleState::puzzle_goto_open(self)
    }
    fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        PuzzleState::selected_analysis_node(self)
    }
    fn settings(&self) -> &WorkspaceSettings {
        PuzzleState::settings(self)
    }
    fn settings_open(&self) -> bool {
        PuzzleState::settings_open(self)
    }
    fn transient_message(&self) -> Option<&str> {
        PuzzleState::transient_message(self)
    }
}
impl ScreenState for ReviewApp {
    fn active_review_game(&self) -> Option<&ReviewGameEntry> {
        ReviewApp::active_review_game(self)
    }
    fn analysis_page(&self) -> usize {
        self.review_state().map_or(0, ReviewState::analysis_page)
    }
    fn board(&self) -> &Board {
        ReviewApp::board(self).unwrap_or_else(|| empty_board())
    }
    fn flipped(&self) -> bool {
        self.review_state().is_some_and(ReviewState::flipped)
    }
    fn mode(&self) -> BoardMode {
        if self
            .review_state()
            .is_some_and(ReviewState::free_board_enabled)
        {
            BoardMode::FreeBoard
        } else {
            BoardMode::Solution
        }
    }
    fn orientation_locked(&self) -> bool {
        self.review_state()
            .is_some_and(ReviewState::orientation_locked)
    }
    fn pending_promotion(&self) -> Option<&PendingPromotion> {
        ReviewApp::pending_promotion(self)
    }
    fn review_picker_entry_count(&self) -> usize {
        ReviewApp::review_picker_entry_count(self)
    }
    fn review_picker_error(&self, picker_index: usize) -> Option<&ReviewFileError> {
        ReviewApp::review_picker_error(self, picker_index)
    }
    fn review_picker_game(&self, picker_index: usize) -> Option<&ReviewGameEntry> {
        ReviewApp::review_picker_game(self, picker_index)
    }
    fn review_state(&self) -> Option<&ReviewState> {
        ReviewApp::review_state(self)
    }
    fn selected_analysis_node(&self) -> Option<AnalysisNodeIndex> {
        self.review_state().map(ReviewState::selected_node)
    }
    fn settings(&self) -> &WorkspaceSettings {
        ReviewApp::settings(self)
    }
    fn settings_open(&self) -> bool {
        ReviewApp::settings_open(self)
    }
    fn transient_message(&self) -> Option<&str> {
        ReviewApp::transient_message(self)
    }
    fn workspace(&self) -> Workspace {
        Workspace::Review
    }
}
fn empty_board() -> &'static Board {
    static EMPTY: std::sync::OnceLock<Board> = std::sync::OnceLock::new();
    EMPTY.get_or_init(|| Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").expect("empty board FEN"))
}
