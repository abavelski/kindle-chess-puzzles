use chess_core::{
    parse_puzzle_file, parse_uci_move, Action, ActiveCollection, AppState, Board, BoardInputStatus,
    BoardIntent, BoardSessionStatus as Status, Effect, PhysicalBoardSeat as Seat, Progress,
    Settings,
};
fn position(value: &str) -> Board {
    Board::from_fen(&format!("{value} w - - 0 1")).unwrap()
}
fn app(seat: Seat, black: bool) -> AppState {
    let data = format!(
        r#"{{"version":1,"puzzles":[{{"id":"seat","fen":"7k/8/8/8/8/8/P6p/K7 {} - - 0 1","solution":{}}}]}}"#,
        if black { "b" } else { "w" },
        if black {
            r#"["h2h1q","a2a3","h1h2"]"#
        } else {
            r#"["a2a3","h2h1q","a3a4"]"#
        }
    );
    let mut settings = Settings::default();
    settings.set_physical_board_seat(seat);
    AppState::new_with_settings(
        ActiveCollection::from_collection("seat.json", parse_puzzle_file(data.as_bytes()).unwrap()),
        Progress::new(),
        settings,
    )
}
#[test]
fn seating_table_rotation_and_settings_compatibility() {
    assert_eq!(
        Settings::parse(br#"{"version":1}"#)
            .unwrap()
            .physical_board_seat(),
        Seat::Changing
    );
    for (seat, white, black) in [
        (Seat::White, false, true),
        (Seat::Black, true, false),
        (Seat::Changing, false, false),
    ] {
        assert_eq!(seat.rotates(chess_core::Color::White), white);
        assert_eq!(seat.rotates(chess_core::Color::Black), black);
        let mut settings = Settings::default();
        settings.set_physical_board_seat(seat);
        assert_eq!(
            Settings::parse(&settings.to_bytes().unwrap()).unwrap(),
            settings
        );
    }
    assert!(Settings::parse(br#"{"version":1,"physical_board_seat":"sideways"}"#).is_err());
    let before = position("7k/8/8/8/8/P7/8/4K3");
    assert_eq!(before.placement_rotated(true), "3K4/8/7P/8/8/8/8/k7");
    assert_eq!(
        position(&before.placement_rotated(true)).placement_rotated(true),
        before.placement()
    );
}
#[test]
fn all_seats_round_trip_solver_input_promotion_and_automatic_reply_without_changing_logical_board()
{
    for seat in [Seat::White, Seat::Black, Seat::Changing] {
        for black in [false, true] {
            let mut app = app(seat, black);
            let rotated = seat.rotates(app.active_puzzle().side_to_move());
            let original = app.board().clone();
            let progress = app.progress().clone();
            app.dispatch(Action::BoardStatus(Status::Connected));
            let wire = original.placement_rotated(rotated);
            assert_eq!(
                app.dispatch(Action::SendBoardPosition),
                vec![Effect::Board(BoardIntent::Send(wire.clone()))]
            );
            assert_eq!(app.board(), &original);
            assert_eq!(app.progress(), &progress);
            app.dispatch(Action::BoardStatus(Status::Connected));
            app.dispatch(Action::BoardPosition(wire));
            assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
            app.dispatch(Action::Flip);
            let mut physical = original.clone();
            physical.apply_uci_move(parse_uci_move(if black { "h2h1q" } else { "a2a3" }).unwrap());
            let effects = app.dispatch(Action::BoardPosition(physical.placement_rotated(rotated)));
            assert_eq!(app.solution_ply(), 2);
            let reply_wire = app.board().placement_rotated(rotated);
            assert_eq!(
                effects,
                vec![Effect::Board(BoardIntent::Send(reply_wire.clone()))]
            );
            assert!(app
                .dispatch(Action::BoardPosition(physical.placement_rotated(rotated)))
                .is_empty());
            app.dispatch(Action::BoardStatus(Status::Connected));
            app.dispatch(Action::BoardPosition(reply_wire));
            assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
            let mut physical = app.board().clone();
            physical.apply_uci_move(parse_uci_move(if black { "h1h2" } else { "a3a4" }).unwrap());
            assert_eq!(
                app.dispatch(Action::BoardPosition(physical.placement_rotated(rotated))),
                vec![Effect::ProgressChanged]
            );
            assert_eq!(app.solution_ply(), 3);
        }
    }
}
#[test]
fn cycling_seat_only_persists_setting_and_requires_a_fresh_send() {
    let mut app = app(Seat::Changing, false);
    app.dispatch(Action::BoardStatus(Status::Connected));
    let wire = app.board().placement();
    app.dispatch(Action::SendBoardPosition);
    app.dispatch(Action::BoardStatus(Status::Connected));
    app.dispatch(Action::BoardPosition(wire));
    app.dispatch(Action::OpenSettings);
    let board = app.board().clone();
    let progress = app.progress().clone();
    let flipped = app.flipped();
    for seat in [Seat::White, Seat::Black, Seat::Changing] {
        assert_eq!(
            app.dispatch(Action::CyclePhysicalBoardSeat),
            vec![Effect::SettingsChanged]
        );
        assert_eq!(app.physical_board_seat(), seat);
        assert_eq!(app.board_input_status(), BoardInputStatus::Off);
        assert_eq!(app.board(), &board);
        assert_eq!(app.progress(), &progress);
        assert_eq!(app.flipped(), flipped);
    }
}

#[test]
fn rotated_capture_and_wrong_move_correction_use_canonical_squares() {
    let data=br#"{"version":1,"puzzles":[{"id":"capture","fen":"7k/8/8/8/1p6/P7/8/K7 w - - 0 1","solution":["a3b4"]}]}"#;
    let mut settings = Settings::default();
    settings.set_physical_board_seat(Seat::Black);
    let mut free = AppState::new_with_settings(
        ActiveCollection::from_collection("capture.json", parse_puzzle_file(data).unwrap()),
        Progress::new(),
        settings,
    );
    free.dispatch(Action::ToggleMode);
    free.dispatch(Action::BoardStatus(Status::Connected));
    let baseline = free.board().placement_rotated(true);
    free.dispatch(Action::SendBoardPosition);
    free.dispatch(Action::BoardStatus(Status::Connected));
    free.dispatch(Action::BoardPosition(baseline));
    let progress = free.progress().clone();
    let mut captured = free.board().clone();
    captured.apply_uci_move(parse_uci_move("a3b4").unwrap());
    assert!(free
        .dispatch(Action::BoardPosition(captured.placement_rotated(true)))
        .is_empty());
    assert_eq!(free.board().placement(), captured.placement());
    assert_eq!(free.progress(), &progress);

    let mut wrong = app(Seat::White, true);
    wrong.dispatch(Action::BoardStatus(Status::Connected));
    let baseline = wrong.board().placement_rotated(true);
    wrong.dispatch(Action::SendBoardPosition);
    wrong.dispatch(Action::BoardStatus(Status::Connected));
    wrong.dispatch(Action::BoardPosition(baseline.clone()));
    let mut moved = wrong.board().clone();
    moved.apply_uci_move(parse_uci_move("h2h3").unwrap());
    assert!(wrong
        .dispatch(Action::BoardPosition(moved.placement_rotated(true)))
        .is_empty());
    assert_eq!(wrong.solution_ply(), 0);
    assert_eq!(wrong.board_input_status(), BoardInputStatus::MatchKindle);
    wrong.dispatch(Action::BoardPosition(baseline));
    assert_eq!(wrong.board_input_status(), BoardInputStatus::Listening);
}

#[test]
fn root_solver_color_controls_preview_send_independently_of_flip_and_review_preferences() {
    let rich = parse_puzzle_file(include_bytes!(
        "../../../tests/fixtures/rich-analysis/valid-rich.json"
    ))
    .unwrap();
    let mut settings = Settings::default();
    settings.set_physical_board_seat(Seat::White);
    let mut preview = AppState::new_with_settings(
        ActiveCollection::from_collection("rich.json", rich),
        Progress::new(),
        settings,
    );
    assert_eq!(
        preview.active_puzzle().side_to_move(),
        chess_core::Color::Black
    );
    preview.dispatch(Action::OpenAnalysis);
    let first = preview
        .active_puzzle()
        .analysis
        .as_ref()
        .unwrap()
        .root()
        .children[0];
    preview.dispatch(Action::SelectAnalysisNode(first));
    preview.dispatch(Action::Flip);
    preview.dispatch(Action::BoardStatus(Status::Connected));
    let expected = preview.board().placement_rotated(true);
    let progress = preview.progress().clone();
    assert_eq!(
        preview.dispatch(Action::SendBoardPosition),
        vec![Effect::Board(BoardIntent::Send(expected))]
    );
    assert_eq!(preview.board_input_status(), BoardInputStatus::Off);
    assert_eq!(preview.progress(), &progress);

    let collection = chess_core::parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap();
    preview.set_review_games(vec![chess_core::ReviewGameEntry::new(
        "games.json",
        collection.games[0].clone(),
    )]);
    preview.dispatch(Action::ToggleWorkspace);
    preview.dispatch(Action::OpenSettings);
    let persisted = preview.persisted_settings().clone();
    assert!(preview.dispatch(Action::CyclePhysicalBoardSeat).is_empty());
    assert_eq!(preview.persisted_settings(), &persisted);
}
