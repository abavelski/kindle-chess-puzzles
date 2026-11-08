use chess_core::{
    puzzle_app::{PuzzleAction, PuzzleState},
    *,
};

#[test]
fn puzzle_product_constructs_and_grades_without_review_library_or_actions() {
    let mut app = PuzzleState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    app.dispatch(PuzzleAction::TapSquare(algebraic_to_square("d7").unwrap()));
    app.dispatch(PuzzleAction::TapSquare(algebraic_to_square("e8").unwrap()));
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    assert!(app.is_current_solved());
}

#[test]
fn product_promotions_free_analysis_settings_and_board_intents_preserve_contracts() {
    use chess_core::puzzle_app::{PuzzleAction as A, PuzzleEffect as E};
    let mut app = PuzzleState::new(
        ActiveCollection::from_collection(
            "rich.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/rich-analysis/valid-rich.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    );
    let before = app.board().clone();
    app.dispatch(A::TapSquare(algebraic_to_square("h2").unwrap()));
    app.dispatch(A::TapSquare(algebraic_to_square("h1").unwrap()));
    assert!(app.pending_promotion().is_some());
    app.dispatch(A::ChoosePromotion(PromotionChoice::Rook));
    assert_eq!(app.board(), &before);
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);
    app.dispatch(A::OpenAnalysis);
    let alternative = app
        .active_puzzle()
        .analysis
        .as_ref()
        .unwrap()
        .node_index("n2")
        .unwrap();
    app.dispatch(A::SelectAnalysisNode(alternative));
    assert_ne!(app.board(), app.live_board());
    let progress = app.progress().to_bytes().unwrap();
    app.dispatch(A::CloseAnalysis);
    app.dispatch(A::ToggleMode);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    app.dispatch(A::OpenSettings);
    assert_eq!(
        app.dispatch(A::ToggleFreeModeSetting),
        vec![E::SettingsChanged]
    );
    assert_eq!(app.mode(), BoardMode::Solution);
    assert_eq!(
        app.dispatch(A::CyclePhysicalBoardSeat),
        vec![E::SettingsChanged]
    );
    assert_eq!(app.physical_board_seat(), PhysicalBoardSeat::White);
    app.dispatch(A::CloseSettings);
    assert_eq!(
        app.dispatch(A::ToggleBoardConnection),
        vec![E::Board(BoardIntent::Connect)]
    );
    app.dispatch(A::BoardStatus(BoardSessionStatus::Connected));
    let sent = app.board().placement_rotated(true);
    assert_eq!(
        app.dispatch(A::SendBoardPosition),
        vec![E::Board(BoardIntent::Send(sent.clone()))]
    );
    app.dispatch(A::BoardStatus(BoardSessionStatus::Connected));
    app.dispatch(A::BoardPosition(sent));
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    app.dispatch(A::PauseBoardInput);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
    assert_eq!(app.progress().to_bytes().unwrap(), progress);
    assert_eq!(app.dispatch(A::Exit), vec![E::ExitRequested]);
}
