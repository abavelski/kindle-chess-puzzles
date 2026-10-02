use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, BoardMode,
    CollectionEntry, Color, Effect, Piece, PieceKind, Progress, PromotionChoice, SolutionFeedback,
};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const PROMOTIONS: &[u8] = include_bytes!("../../../tests/fixtures/promotion-puzzles.json");

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn state(key: &str, bytes: &[u8]) -> AppState {
    let collection = parse_puzzle_file(bytes).expect("test collection parses");
    AppState::new(
        ActiveCollection::from_collection(key, collection),
        Progress::new(),
    )
}

fn play(app: &mut AppState, movement: &str) -> Vec<Effect> {
    let from = square(&movement[..2]);
    let to = square(&movement[2..4]);
    let mut effects = app.dispatch(Action::TapSquare(from));
    effects.extend(app.dispatch(Action::TapSquare(to)));
    if movement.len() == 5 {
        let choice = match movement.as_bytes()[4] {
            b'q' => PromotionChoice::Queen,
            b'r' => PromotionChoice::Rook,
            b'b' => PromotionChoice::Bishop,
            b'n' => PromotionChoice::Knight,
            _ => panic!("invalid test promotion"),
        };
        effects.extend(app.dispatch(Action::ChoosePromotion(choice)));
    }
    effects
}

fn piece(color: Color, kind: PieceKind) -> Option<Piece> {
    Some(Piece { color, kind })
}

#[test]
fn selection_deselection_and_wrong_moves_do_not_advance_solution() {
    let mut app = state("puzzles.json", PUZZLES);
    let initial = app.board().clone();

    assert!(app.dispatch(Action::TapSquare(square("d7"))).is_empty());
    assert_eq!(app.board().selected(), Some(square("d7")));
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);

    app.dispatch(Action::TapSquare(square("d7")));
    assert_eq!(app.board().selected(), None);
    assert_eq!(app.solution_ply(), 0);

    play(&mut app, "d7d8");
    assert_eq!(app.board(), &initial);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);
}

#[test]
fn one_move_solution_completes_marks_solved_once_and_ignores_more_board_taps() {
    let mut app = state("puzzles.json", PUZZLES);
    let effects = play(&mut app, "d7e8");

    assert_eq!(app.solution_ply(), 1);
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    assert!(app.description_visible());
    assert!(app.is_current_solved());
    assert_eq!(effects, vec![Effect::ProgressChanged]);

    let solved = app.board().clone();
    app.dispatch(Action::TapSquare(square("e8")));
    app.dispatch(Action::TapSquare(square("e7")));
    assert_eq!(app.board(), &solved);

    app.dispatch(Action::Reset);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert_eq!(app.solution_ply(), 0);
    assert!(!app.description_visible());
    let repeat_effects = play(&mut app, "d7e8");
    assert!(
        repeat_effects.is_empty(),
        "already-solved puzzle is not re-added"
    );
}

#[test]
fn three_ply_line_auto_applies_coordinate_reply_then_accepts_final_move() {
    let mut app = state("puzzles.json", PUZZLES);
    app.dispatch(Action::NextPuzzle);

    play(&mut app, "e2e6");
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(app.feedback(), SolutionFeedback::Correct);
    assert_eq!(app.board().piece_at(square("f7")), None);
    assert_eq!(
        app.board().piece_at(square("f8")),
        piece(Color::Black, PieceKind::King)
    );

    play(&mut app, "e6f7");
    assert_eq!(app.solution_ply(), 3);
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    assert!(app.is_current_solved());
}

#[test]
fn automatic_opponent_promotion_applies_suffix_without_opening_promotion_state() {
    let mut app = state("promotions.json", PROMOTIONS);
    for _ in 0..3 {
        app.dispatch(Action::NextPuzzle);
    }

    play(&mut app, "h2h3");

    assert!(app.pending_promotion().is_none());
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(app.feedback(), SolutionFeedback::Correct);
    assert_eq!(
        app.board().piece_at(square("a1")),
        piece(Color::Black, PieceKind::Queen)
    );
}

#[test]
fn promotion_is_staged_cancelled_and_graded_atomically_for_both_colors() {
    let mut app = state("promotions.json", PROMOTIONS);
    let initial = app.board().clone();

    app.dispatch(Action::TapSquare(square("a7")));
    app.dispatch(Action::TapSquare(square("a8")));
    let pending = app.pending_promotion().expect("white promotion is pending");
    assert_eq!(pending.from(), square("a7"));
    assert_eq!(pending.to(), square("a8"));
    assert_eq!(pending.color(), Color::White);
    assert_eq!(
        app.board().piece_at(square("a7")),
        initial.piece_at(square("a7"))
    );
    assert_eq!(
        app.board().piece_at(square("a8")),
        initial.piece_at(square("a8"))
    );

    app.dispatch(Action::CancelPromotion);
    assert_eq!(app.board(), &initial);
    assert!(app.pending_promotion().is_none());
    assert_eq!(app.solution_ply(), 0);

    app.dispatch(Action::NextPuzzle);
    let knight_start = app.board().clone();
    play(&mut app, "b7b8q");
    assert_eq!(app.board(), &knight_start);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);

    play(&mut app, "b7b8n");
    assert_eq!(
        app.board().piece_at(square("b8")),
        piece(Color::White, PieceKind::Knight)
    );
    assert_eq!(app.feedback(), SolutionFeedback::Complete);

    app.dispatch(Action::NextPuzzle);
    play(&mut app, "h2h1r");
    assert_eq!(
        app.board().piece_at(square("h1")),
        piece(Color::Black, PieceKind::Rook)
    );
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
}

#[test]
fn navigation_and_reset_clear_attempt_description_and_pending_promotion() {
    let mut app = state("promotions.json", PROMOTIONS);
    app.dispatch(Action::ToggleDescription);
    app.dispatch(Action::TapSquare(square("a7")));
    app.dispatch(Action::TapSquare(square("a8")));
    assert!(app.pending_promotion().is_some());
    assert!(app.description_visible());

    app.dispatch(Action::NextPuzzle);
    assert_eq!(app.active_puzzle_index(), 1);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert!(app.pending_promotion().is_none());
    assert!(!app.description_visible());

    play(&mut app, "b7b8n");
    assert_eq!(app.feedback(), SolutionFeedback::Complete);
    app.dispatch(Action::Reset);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert!(!app.description_visible());
}

#[test]
fn free_board_is_ungraded_and_mode_transitions_preserve_the_required_state() {
    let mut app = state("puzzles.json", PUZZLES);
    app.dispatch(Action::NextPuzzle);
    play(&mut app, "e2e6");
    let line_position = app.board().clone();
    assert_eq!(app.solution_ply(), 2);

    app.dispatch(Action::ToggleMode);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    assert_eq!(app.board(), &line_position);
    assert_eq!(app.solution_ply(), 2);
    assert_eq!(app.feedback(), SolutionFeedback::None);

    play(&mut app, "e6e5");
    assert_eq!(app.solution_ply(), 2);
    assert!(!app.is_current_solved());

    app.dispatch(Action::Flip);
    let flipped = app.flipped();
    app.dispatch(Action::ToggleMode);
    assert_eq!(app.mode(), BoardMode::Solution);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert_eq!(app.flipped(), flipped);

    app.dispatch(Action::ToggleMode);
    play(&mut app, "e2e5");
    app.dispatch(Action::Reset);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    app.dispatch(Action::PreviousPuzzle);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
}

#[test]
fn free_board_promotion_never_marks_solved() {
    let mut app = state("promotions.json", PROMOTIONS);
    app.dispatch(Action::ToggleMode);
    let effects = play(&mut app, "a7a8r");

    assert!(effects.is_empty());
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert!(!app.is_current_solved());
    assert_eq!(
        app.board().piece_at(square("a8")),
        piece(Color::White, PieceKind::Rook)
    );
}

#[test]
fn orientation_lock_flip_unlock_and_navigation_follow_reference_rules() {
    let black_first = br#"{
      "version": 1,
      "title": "Orientation",
      "puzzles": [
        {"id":"white","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","solution":["g6g7"]},
        {"id":"black","fen":"8/8/8/8/8/5kq1/8/7K b - - 0 1","solution":["g3g2"]},
        {"id":"white2","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","solution":["g6g7"]}
      ]
    }"#;
    let mut app = state("orientation.json", black_first);
    assert!(!app.flipped());

    app.dispatch(Action::ToggleOrientationLock);
    app.dispatch(Action::NextPuzzle);
    assert!(app.orientation_locked());
    assert!(!app.flipped(), "locked orientation is preserved");

    app.dispatch(Action::Flip);
    assert!(app.flipped());
    app.dispatch(Action::NextPuzzle);
    assert!(app.flipped(), "manual flip is preserved while locked");

    app.dispatch(Action::ToggleOrientationLock);
    assert!(!app.orientation_locked());
    assert!(app.flipped(), "unlock does not jump immediately");
    app.dispatch(Action::PreviousPuzzle);
    assert!(app.flipped(), "black puzzle auto-orients to black");
    app.dispatch(Action::PreviousPuzzle);
    assert!(
        !app.flipped(),
        "later navigation resumes white auto-orientation"
    );
}

#[test]
fn description_toggle_does_not_solve_and_completion_reveals_it() {
    let mut app = state("puzzles.json", PUZZLES);
    app.dispatch(Action::ToggleDescription);
    assert!(app.description_visible());
    assert!(!app.is_current_solved());
    assert_eq!(app.solution_ply(), 0);

    app.dispatch(Action::ToggleDescription);
    assert!(!app.description_visible());
    play(&mut app, "d7e8");
    assert!(app.description_visible());
    assert!(app.is_current_solved());
    app.dispatch(Action::ToggleDescription);
    assert!(!app.description_visible());
}

#[test]
fn collection_activation_restores_remembered_id_updates_progress_and_respects_lock() {
    let first = parse_puzzle_file(PUZZLES).expect("first collection");
    let second_bytes = br#"{
      "version": 1,
      "title": "Second",
      "puzzles": [
        {"id":"second-white","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","solution":["g6g7"]},
        {"id":"second-black","fen":"8/8/8/8/8/5kq1/8/7K b - - 0 1","solution":["g3g2"]}
      ]
    }"#;
    let second = parse_puzzle_file(second_bytes).expect("second collection");
    let mut progress = Progress::new();
    progress.remember_puzzle("second.json", "second-black");
    let mut app = AppState::new(
        ActiveCollection::from_collection("first.json", first),
        progress,
    );

    app.dispatch(Action::ToggleOrientationLock);
    app.dispatch(Action::Flip);
    let effects = app.dispatch(Action::ActivateCollection(
        ActiveCollection::from_collection("second.json", second),
    ));

    assert_eq!(app.active_collection().key(), "second.json");
    assert_eq!(app.active_collection().title(), Some("Second"));
    assert_eq!(app.active_puzzle().id, "second-black");
    assert!(app.orientation_locked());
    assert!(app.flipped());
    assert_eq!(effects, vec![Effect::ProgressChanged]);
    assert_eq!(app.progress().active_file.as_deref(), Some("second.json"));
}

#[test]
fn invalid_stored_reply_restores_attempt_and_surfaces_transient_error() {
    let invalid_reply = br#"{
      "version": 1,
      "puzzles": [{
        "id":"bad-reply",
        "fen":"8/3B2pp/p5k1/6P1/1ppp1K2/8/1P6/8 w - - 0 39",
        "solution":["d7e8","a1a2","e8d7"]
      }]
    }"#;
    let mut app = state("bad.json", invalid_reply);
    let initial = app.board().clone();

    play(&mut app, "d7e8");

    assert_eq!(app.board(), &initial);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.feedback(), SolutionFeedback::None);
    assert!(app
        .transient_message()
        .is_some_and(|message| message.contains("a1a2")));

    app.dispatch(Action::SetTransientMessage(None));
    assert!(app.transient_message().is_none());
}

#[test]
fn navigation_availability_matches_collection_ends_without_wrap() {
    let mut app = state("puzzles.json", PUZZLES);

    assert!(!app.can_previous_puzzle());
    assert!(app.can_next_puzzle());
    let first = app.board().clone();
    app.dispatch(Action::PreviousPuzzle);
    assert_eq!(app.active_puzzle_index(), 0);
    assert_eq!(app.board(), &first);

    app.dispatch(Action::NextPuzzle);
    assert!(app.can_previous_puzzle());
    assert!(!app.can_next_puzzle());
    let last = app.board().clone();
    app.dispatch(Action::NextPuzzle);
    assert_eq!(app.active_puzzle_index(), 1);
    assert_eq!(app.board(), &last);
}


#[test]
fn collection_picker_requests_a_file_without_mutating_the_active_board() {
    let mut app = state("puzzles.json", PUZZLES);
    let initial_board = app.board().clone();
    app.set_collection_entries(vec![
        CollectionEntry::valid("puzzles.json", "Main"),
        CollectionEntry::invalid("puzzles-broken.json", "invalid JSON"),
        CollectionEntry::valid("puzzles-endgames.json", "Endgames"),
    ]);

    app.dispatch(Action::OpenCollectionPicker);
    assert!(app.collection_picker_open());
    assert_eq!(
        app.dispatch(Action::SelectCollection(1)),
        vec![Effect::CollectionRequested("puzzles-broken.json".to_owned())]
    );
    assert_eq!(app.active_collection().key(), "puzzles.json");
    assert_eq!(app.board(), &initial_board);

    app.dispatch(Action::TapSquare(square("d7")));
    assert_eq!(app.board(), &initial_board, "picker blocks ordinary board actions");

    app.dispatch(Action::CloseCollectionPicker);
    assert!(!app.collection_picker_open());
}

#[test]
fn collection_picker_pages_catalogs_larger_than_one_scribe_page() {
    let mut app = state("puzzles.json", PUZZLES);
    app.set_collection_entries(
        (0..7)
            .map(|index| {
                CollectionEntry::valid(
                    format!("puzzles-{index}.json"),
                    format!("Collection {index}"),
                )
            })
            .collect(),
    );

    app.dispatch(Action::OpenCollectionPicker);
    assert_eq!(app.collection_picker_page_count(), 2);
    assert_eq!(app.collection_picker_visible_range(), 0..6);
    assert!(!app.collection_picker_can_previous_page());
    assert!(app.collection_picker_can_next_page());

    app.dispatch(Action::CollectionPickerNextPage);
    assert_eq!(app.collection_picker_visible_range(), 6..7);
    assert!(app.collection_picker_can_previous_page());
    assert!(!app.collection_picker_can_next_page());
}

#[test]
fn restart_restores_active_file_current_puzzle_and_solved_ids_per_file() {
    let first = parse_puzzle_file(PUZZLES).expect("first collection");
    let second = parse_puzzle_file(PROMOTIONS).expect("second collection");
    let mut app = AppState::new(
        ActiveCollection::from_collection("puzzles.json", first.clone()),
        Progress::new(),
    );

    play(&mut app, "d7e8");
    assert!(app
        .progress()
        .is_solved("puzzles.json", "lichess-001cr"));
    app.dispatch(Action::NextPuzzle);
    assert_eq!(app.active_puzzle().id, "lichess-000hf");

    app.dispatch(Action::ActivateCollection(
        ActiveCollection::from_collection("puzzles-promotions.json", second.clone()),
    ));
    play(&mut app, "a7a8q");
    assert!(app
        .progress()
        .is_solved("puzzles-promotions.json", "promotion-white-queen"));
    app.dispatch(Action::NextPuzzle);
    assert_eq!(app.active_puzzle_index(), 1);

    let saved = app.progress().clone();
    assert_eq!(
        saved.active_file.as_deref(),
        Some("puzzles-promotions.json")
    );

    let mut restarted = AppState::new(
        ActiveCollection::from_collection("puzzles-promotions.json", second),
        saved,
    );
    assert_eq!(restarted.active_puzzle_index(), 1);
    assert!(restarted
        .progress()
        .is_solved("puzzles.json", "lichess-001cr"));
    assert!(restarted
        .progress()
        .is_solved("puzzles-promotions.json", "promotion-white-queen"));

    restarted.dispatch(Action::ActivateCollection(
        ActiveCollection::from_collection("puzzles.json", first),
    ));
    assert_eq!(restarted.active_puzzle().id, "lichess-000hf");
}
