use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, BoardIntent,
    BoardSessionStatus as Status, Effect, Progress,
};
fn app() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    )
}
#[test]
fn direct_send_snapshots_once_without_modal_or_progress_changes() {
    let mut app = app();
    let progress = app.progress().clone();
    let settings = app.settings().clone();
    assert!(!app.can_send_position());
    app.dispatch(Action::BoardStatus(Status::Connected));
    let target = app.board().placement();
    assert_eq!(
        app.dispatch(Action::SendBoardPosition),
        vec![Effect::Board(BoardIntent::Send(target.clone()))]
    );
    assert_eq!(app.board_target(), Some(target.as_str()));
    assert!(app.board_status().busy());
    assert!(!app.can_send_position());
    assert!(app.dispatch(Action::SendBoardPosition).is_empty());
    app.dispatch(Action::BoardStatus(Status::Sending));
    let flipped = app.flipped();
    app.dispatch(Action::Flip);
    assert_ne!(
        app.flipped(),
        flipped,
        "arranging must not block app navigation"
    );
    assert_eq!(app.board_target(), Some(target.as_str()));
    app.dispatch(Action::BoardStatus(Status::Connected));
    assert!(app.board_target().is_none());
    assert!(app.can_send_position());
    assert_eq!(app.progress(), &progress);
    assert_eq!(app.settings(), &settings);
}
#[test]
fn disconnect_is_available_during_arranging_and_keeps_no_authorized_target() {
    let mut app = app();
    app.dispatch(Action::BoardStatus(Status::Connected));
    app.dispatch(Action::SendBoardPosition);
    app.dispatch(Action::BoardStatus(Status::Sending));
    app.dispatch(Action::OpenSettings);
    assert!(app.settings_open());
    assert_eq!(app.board_status().connection_control_label(), "DISCONNECT");
    assert_eq!(
        app.dispatch(Action::ToggleBoardConnection),
        vec![Effect::Board(BoardIntent::Disconnect)]
    );
    assert!(app.board_target().is_none());
    assert!(!app.can_send_position());
}
#[test]
fn source_status_updates_pass_through_settings_modal() {
    let mut app = app();
    app.dispatch(Action::OpenSettings);
    assert_eq!(
        app.dispatch(Action::ToggleBoardConnection),
        vec![Effect::Board(BoardIntent::Connect)]
    );
    app.dispatch(Action::BoardStatus(Status::Connected));
    assert_eq!(app.board_status(), &Status::Connected);
    assert_eq!(
        app.dispatch(Action::ToggleBoardConnection),
        vec![Effect::Board(BoardIntent::Disconnect)]
    );
}

#[test]
fn displayed_preview_and_free_board_are_captured_without_progress_or_orientation_changes() {
    for free in [false, true] {
        let data = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
        let mut app = AppState::new(
            ActiveCollection::from_collection("rich.json", parse_puzzle_file(data).unwrap()),
            Progress::new(),
        );
        let initial = app.board().placement();
        if free {
            app.dispatch(Action::ToggleMode);
            app.dispatch(Action::TapSquare(
                chess_core::algebraic_to_square("h2").unwrap(),
            ));
            app.dispatch(Action::TapSquare(
                chess_core::algebraic_to_square("h3").unwrap(),
            ));
        } else {
            app.dispatch(Action::OpenAnalysis);
            app.dispatch(Action::SelectAnalysisNode(
                app.active_puzzle()
                    .analysis
                    .as_ref()
                    .unwrap()
                    .node_index("n1")
                    .unwrap(),
            ));
        }
        let before = app.clone();
        app.dispatch(Action::Flip);
        app.dispatch(Action::BoardStatus(Status::Connected));
        let visible = app.board().placement();
        assert_ne!(
            visible, initial,
            "fixture must exercise a displayed/scratch position"
        );
        assert_eq!(
            app.dispatch(Action::SendBoardPosition),
            vec![Effect::Board(BoardIntent::Send(visible))]
        );
        assert_eq!(app.progress(), before.progress());
        assert_eq!(app.live_board(), before.live_board());
        assert_eq!(app.active_collection(), before.active_collection());
    }
}
#[test]
fn review_and_pending_promotion_cannot_send_and_ordinary_actions_never_send() {
    let mut app = app();
    let games = chess_core::parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap();
    app.set_review_games(vec![chess_core::ReviewGameEntry::new(
        "games.json",
        games.games[0].clone(),
    )]);
    app.dispatch(Action::BoardStatus(Status::Connected));
    app.dispatch(Action::ToggleWorkspace);
    let review = app.review_resume();
    assert!(!app.can_send_position());
    assert!(app.dispatch(Action::SendBoardPosition).is_empty());
    assert_eq!(app.review_resume(), review);
    app.dispatch(Action::ToggleWorkspace);
    for action in [
        Action::NextPuzzle,
        Action::PreviousPuzzle,
        Action::Reset,
        Action::Flip,
    ] {
        assert!(!app
            .dispatch(action)
            .iter()
            .any(|e| matches!(e, Effect::Board(_))));
    }
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "p.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/promotion-puzzles.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    );
    app.dispatch(Action::BoardStatus(Status::Connected));
    app.dispatch(Action::TapSquare(
        chess_core::algebraic_to_square("a7").unwrap(),
    ));
    app.dispatch(Action::TapSquare(
        chess_core::algebraic_to_square("a8").unwrap(),
    ));
    assert!(app.pending_promotion().is_some());
    assert!(app.dispatch(Action::SendBoardPosition).is_empty());
}

#[test]
fn canonical_placement_matches_authored_coordinates() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/puzzles.json").as_slice(),
        include_bytes!("../../../tests/fixtures/promotion-puzzles.json").as_slice(),
    ] {
        for puzzle in parse_puzzle_file(bytes).unwrap().puzzles {
            assert_eq!(
                chess_core::Board::from_fen(&puzzle.fen)
                    .unwrap()
                    .placement(),
                puzzle.fen.split_whitespace().next().unwrap()
            );
        }
    }
}

#[test]
fn transport_submission_acknowledgement_allows_another_explicit_send() {
    let mut app = app();
    app.dispatch(Action::BoardStatus(Status::Connected));
    for _ in 0..2 {
        assert!(matches!(
            app.dispatch(Action::SendBoardPosition).as_slice(),
            [Effect::Board(BoardIntent::Send(_))]
        ));
        assert_eq!(app.board_status(), &Status::Sending);
        assert!(app.dispatch(Action::SendBoardPosition).is_empty());
        app.dispatch(Action::BoardStatus(Status::Connected));
        assert!(app.board_target().is_none());
        assert!(app.can_send_position());
    }
}
