use chess_core::{
    infer_board_move, parse_puzzle_file, parse_uci_move, Action, ActiveCollection, AppState, Board,
    BoardInputStatus, BoardMode, BoardSessionStatus as Status, Effect, Progress, SolutionFeedback,
};
fn board(s: &str) -> Board {
    Board::from_fen(&format!("{s} w - - 0 1")).unwrap()
}
fn app() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    )
}
fn sent(app: &mut AppState) -> String {
    app.dispatch(Action::BoardStatus(Status::Connected));
    let target = app.board().placement();
    app.dispatch(Action::SendBoardPosition);
    app.dispatch(Action::BoardStatus(Status::Connected));
    target
}
fn observe(app: &mut AppState, value: &str) -> Vec<Effect> {
    app.dispatch(Action::BoardPosition(value.into()))
}
#[test]
fn physical_move_inference_ignores_lifts_capture_removal_and_requires_exact_final_shape() {
    for (before, after, expected) in [
        ("8/8/8/8/8/8/P7/8", "8/8/8/8/P7/8/8/8", Some("a2a4")),
        ("8/8/8/8/1p6/P7/8/8", "8/8/8/8/1P6/8/8/8", Some("a3b4")),
        ("8/8/8/8/1p6/P7/8/8", "8/8/8/8/8/P7/8/8", None),
        ("8/8/8/8/8/8/P7/8", "8/8/8/8/8/8/8/8", None),
        ("8/P7/8/8/8/8/8/8", "N7/8/8/8/8/8/8/8", Some("a7a8n")),
        ("8/P7/8/8/8/8/8/8", "P7/8/8/8/8/8/8/8", None),
        ("8/8/8/8/8/8/8/4K2R", "8/8/8/8/8/8/8/6KR", None),
        ("8/8/8/3pP3/8/8/8/8", "8/8/3P4/3p4/8/8/8/8", None),
    ] {
        assert_eq!(
            infer_board_move(&board(before), &board(after))
                .map(|m| m.to_string())
                .as_deref(),
            expected
        );
    }
}
#[test]
fn only_after_sent_target_is_seen_a_physical_solver_move_grades_and_persists_once() {
    let mut app = app();
    let baseline = app.board().placement();
    let mut moved = app.board().clone();
    moved.apply_uci_move(parse_uci_move("d7e8").unwrap());
    assert!(observe(&mut app, &moved.placement()).is_empty());
    assert_eq!(app.solution_ply(), 0);
    sent(&mut app);
    assert_eq!(
        app.board_input_status(),
        BoardInputStatus::WaitingForPosition
    );
    observe(&mut app, &moved.placement());
    assert_eq!(app.solution_ply(), 0);
    observe(&mut app, &baseline);
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    let effects = observe(&mut app, &moved.placement());
    assert!(
        !effects.is_empty(),
        "normal solved progress effect must reach runtime"
    );
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    assert_eq!(app.solution_ply(), 1);
    assert!(observe(&mut app, &moved.placement()).is_empty());
    assert_eq!(app.solution_ply(), 1);
}
#[test]
fn wrong_move_requires_physical_correction_and_auto_reply_echo_does_not_grade() {
    let mut app = app();
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    let mut wrong = app.board().clone();
    wrong.apply_uci_move(parse_uci_move("d7f5").unwrap());
    observe(&mut app, &wrong.placement());
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);
    assert_eq!(app.board_input_status(), BoardInputStatus::MatchKindle);
    let mut correct = app.board().clone();
    correct.apply_uci_move(parse_uci_move("d7e8").unwrap());
    observe(&mut app, &correct.placement());
    assert_eq!(app.solution_ply(), 0);
    observe(&mut app, &baseline);
    observe(&mut app, &correct.placement());
    assert_eq!(app.solution_ply(), 1);

    app.dispatch(Action::NextPuzzle);
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    let mut physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e2e6").unwrap());
    let effects = observe(&mut app, &physical.placement());
    assert!(matches!(
        effects.as_slice(),
        [Effect::Board(chess_core::BoardIntent::Send(_))]
    ));
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(
        app.board_input_status(),
        BoardInputStatus::WaitingForPosition
    );
    app.dispatch(Action::BoardStatus(Status::Connected));
    observe(&mut app, &physical.placement());
    assert_eq!(app.solution_ply(), 2);
    let logical = app.board().placement();
    observe(&mut app, &logical);
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e6f7").unwrap());
    observe(&mut app, &physical.placement());
    assert_eq!(app.solution_ply(), 3);
}
#[test]
fn free_board_input_is_orientation_independent_and_navigation_or_sleep_disarms_it() {
    let mut app = app();
    app.dispatch(Action::ToggleMode);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    let progress = app.progress().clone();
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    app.dispatch(Action::Flip);
    app.dispatch(Action::OpenSettings);
    let mut physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("d7e8").unwrap());
    assert!(observe(&mut app, &physical.placement()).is_empty());
    assert_eq!(app.board().placement(), physical.placement());
    assert_eq!(app.progress(), &progress);
    app.dispatch(Action::CloseSettings);
    app.dispatch(Action::Reset);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
    sent(&mut app);
    app.dispatch(Action::PauseBoardInput);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
}

#[test]
fn preview_review_manual_input_and_errors_disarm_and_promotion_is_atomic() {
    let mut app = app();
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    app.dispatch(Action::TapSquare(
        chess_core::algebraic_to_square("d7").unwrap(),
    ));
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
    sent(&mut app);
    app.dispatch(Action::BoardStatus(Status::Error("lost".into())));
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
    let rich = parse_puzzle_file(include_bytes!(
        "../../../tests/fixtures/rich-analysis/valid-rich.json"
    ))
    .unwrap();
    let mut app = AppState::new(
        ActiveCollection::from_collection("rich.json", rich),
        Progress::new(),
    );
    app.dispatch(Action::OpenAnalysis);
    sent(&mut app);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
    app.dispatch(Action::CloseAnalysis);
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);

    let data=br#"{"version":1,"puzzles":[{"id":"promotion","fen":"8/P7/8/8/8/8/8/8 w - - 0 1","solution":["a7a8q"]}]}"#;
    let mut app = AppState::new(
        ActiveCollection::from_collection("promotion.json", parse_puzzle_file(data).unwrap()),
        Progress::new(),
    );
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    observe(&mut app, "P7/8/8/8/8/8/8/8");
    assert_eq!(app.solution_ply(), 0);
    let effects = observe(&mut app, "Q7/8/8/8/8/8/8/8");
    assert_eq!(effects, vec![Effect::ProgressChanged]);
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    assert!(app.pending_promotion().is_none());
}

#[test]
fn rook_first_castling_intermediate_is_not_guessed_as_a_rook_move() {
    assert_eq!(
        infer_board_move(&board("8/8/8/8/8/8/8/4K2R"), &board("8/8/8/8/8/8/8/4KR2")),
        None
    );
}

#[test]
fn hiding_free_board_while_listening_invalidates_input_when_it_returns_to_solution() {
    let mut app = app();
    app.dispatch(Action::ToggleMode);
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    app.dispatch(Action::OpenSettings);
    app.dispatch(Action::ToggleFreeModeSetting);
    assert_eq!(app.mode(), BoardMode::Solution);
    assert_eq!(app.board_input_status(), BoardInputStatus::Off);
}

#[test]
fn accepted_physical_move_submits_one_reply_target_then_waits_for_physical_agreement() {
    use chess_core::BoardIntent;
    let mut app = app();
    app.dispatch(Action::NextPuzzle);
    let baseline = sent(&mut app);
    observe(&mut app, &baseline);
    let mut physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e2e6").unwrap());
    let effects = observe(&mut app, &physical.placement());
    let reply = app.board().placement();
    assert_eq!(
        effects,
        vec![Effect::Board(BoardIntent::Send(reply.clone()))]
    );
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(app.board_status(), &Status::Sending);
    assert_eq!(
        app.board_input_status(),
        BoardInputStatus::WaitingForPosition
    );
    assert!(observe(&mut app, &physical.placement()).is_empty());
    app.dispatch(Action::BoardStatus(Status::Connected));
    assert!(observe(&mut app, &physical.placement()).is_empty());
    assert_eq!(app.solution_ply(), 2);
    assert!(observe(&mut app, &reply).is_empty());
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    assert!(observe(&mut app, &reply).is_empty());
    physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e6f7").unwrap());
    assert_eq!(
        observe(&mut app, &physical.placement()),
        vec![Effect::ProgressChanged]
    );
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
}

#[test]
fn automatic_reply_uses_promoted_target_and_invalid_stored_reply_never_sends() {
    use chess_core::BoardIntent;
    for reply in ["h2h1q", "e8e7"] {
        let data = format!(
            r#"{{"version":1,"puzzles":[{{"id":"reply","fen":"8/8/8/8/8/8/P6p/8 w - - 0 1","solution":["a2a3","{reply}"]}}]}}"#
        );
        let mut app = AppState::new(
            ActiveCollection::from_collection(
                "reply.json",
                parse_puzzle_file(data.as_bytes()).unwrap(),
            ),
            Progress::new(),
        );
        let baseline = sent(&mut app);
        observe(&mut app, &baseline);
        let mut physical = app.board().clone();
        physical.apply_uci_move(parse_uci_move("a2a3").unwrap());
        let effects = observe(&mut app, &physical.placement());
        if reply == "h2h1q" {
            assert_eq!(app.board().placement(), "8/8/8/8/8/P7/8/7q");
            assert_eq!(
                effects,
                vec![
                    Effect::ProgressChanged,
                    Effect::Board(BoardIntent::Send(app.board().placement()))
                ]
            );
        } else {
            assert!(effects.is_empty());
            assert_eq!(app.solution_ply(), 0);
            assert_eq!(app.board_input_status(), BoardInputStatus::MatchKindle);
        }
    }
}
