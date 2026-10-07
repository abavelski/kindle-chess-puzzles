use chess_core::{
    algebraic_to_square, parse_puzzle_file, parse_review_file, Action, ActiveCollection,
    AnalysisNodeIndex, AppState, Board, Effect, Progress, PromotionChoice, ReviewGameEntry,
    SolutionFeedback, Workspace,
};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const RICH: &[u8] = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
const REVIEW: &[u8] = include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
const CUSTOM_REVIEW: &[u8] =
    include_bytes!("../../../tests/fixtures/game-review/valid-custom-fen.json");

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn puzzle_state(bytes: &[u8]) -> AppState {
    let collection = parse_puzzle_file(bytes).expect("puzzle fixture");
    AppState::new(
        ActiveCollection::from_collection("puzzles.json", collection),
        Progress::new(),
    )
}

fn review_entry(collection_id: &str, bytes: &[u8]) -> ReviewGameEntry {
    let collection = parse_review_file(bytes).expect("review fixture");
    ReviewGameEntry::new(collection_id, collection.games[0].clone())
}

fn add_reviews(app: &mut AppState) {
    app.set_review_games(vec![
        review_entry("games-standard.json", REVIEW),
        review_entry("games-custom.json", CUSTOM_REVIEW),
    ]);
}

fn review_index(app: &AppState, id: &str) -> AnalysisNodeIndex {
    app.active_review_game()
        .expect("active review game")
        .game()
        .analysis
        .node_index(id)
        .expect("review node")
}

fn puzzle_analysis_index(app: &AppState, id: &str) -> AnalysisNodeIndex {
    app.active_puzzle()
        .analysis
        .as_ref()
        .expect("puzzle analysis")
        .node_index(id)
        .expect("puzzle analysis node")
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

#[test]
fn entering_review_without_games_stays_in_puzzles_and_surfaces_message() {
    let mut app = puzzle_state(PUZZLES);
    let before = app.live_board().clone();

    assert!(app.dispatch(Action::ToggleWorkspace).is_empty());

    assert_eq!(app.workspace(), Workspace::Puzzles);
    assert_eq!(app.live_board(), &before);
    assert!(app
        .transient_message()
        .is_some_and(|message| message.contains("No review games")));
}

#[test]
fn repeated_workspace_switches_preserve_wrong_attempt_open_analysis_and_orientation() {
    let mut app = puzzle_state(RICH);
    add_reviews(&mut app);
    play(&mut app, "h2h1r");
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);
    app.dispatch(Action::OpenAnalysis);
    let preview = puzzle_analysis_index(&app, "n2");
    app.dispatch(Action::SelectAnalysisNode(preview));
    app.dispatch(Action::AnalysisNextPage);
    app.dispatch(Action::AnalysisNextPage);
    app.dispatch(Action::ToggleOrientationLock);
    app.dispatch(Action::Flip);

    let puzzle_live = app.live_board().clone();
    let puzzle_preview = app.board().clone();
    let puzzle_flipped = app.flipped();
    let progress = app.progress().to_bytes().expect("progress bytes");

    for _ in 0..2 {
        app.dispatch(Action::ToggleWorkspace);
        assert_eq!(app.workspace(), Workspace::Review);
        app.dispatch(Action::ReviewNext);
        app.dispatch(Action::ToggleWorkspace);
        assert_eq!(app.workspace(), Workspace::Puzzles);
    }

    assert_eq!(app.live_board(), &puzzle_live);
    assert_eq!(app.board(), &puzzle_preview);
    assert_eq!(app.feedback(), SolutionFeedback::Wrong);
    assert_eq!(app.solution_ply(), 0);
    assert_eq!(app.selected_analysis_node(), Some(preview));
    assert_eq!(app.analysis_page(), 2);
    assert!(app.orientation_locked());
    assert_eq!(app.flipped(), puzzle_flipped);
    assert_eq!(
        app.progress().to_bytes().expect("unchanged progress"),
        progress
    );
}

#[test]
fn workspace_switch_preserves_in_progress_correct_attempt_and_solved_progress_bytes() {
    let mut correct = puzzle_state(PUZZLES);
    add_reviews(&mut correct);
    correct.dispatch(Action::NextPuzzle);
    play(&mut correct, "e2e6");
    assert_eq!(correct.feedback(), SolutionFeedback::Correct);
    assert_eq!(correct.solution_ply(), 2);
    let correct_board = correct.live_board().clone();

    correct.dispatch(Action::ToggleWorkspace);
    correct.dispatch(Action::ReviewNext);
    correct.dispatch(Action::ToggleWorkspace);
    assert_eq!(correct.feedback(), SolutionFeedback::Correct);
    assert_eq!(correct.solution_ply(), 2);
    assert_eq!(correct.live_board(), &correct_board);

    let mut solved = puzzle_state(PUZZLES);
    add_reviews(&mut solved);
    assert_eq!(play(&mut solved, "d7e8"), vec![Effect::ProgressChanged]);
    let solved_bytes = solved.progress().to_bytes().expect("solved progress");
    solved.dispatch(Action::ToggleWorkspace);
    let n1 = review_index(&solved, "n1");
    assert!(solved.dispatch(Action::SelectAnalysisNode(n1)).is_empty());
    assert!(solved.dispatch(Action::ReviewToggleFree).is_empty());
    assert!(solved.dispatch(Action::ReviewNext).is_empty());
    assert!(solved.dispatch(Action::ReviewReset).is_empty());
    solved.dispatch(Action::ToggleWorkspace);
    assert_eq!(
        solved.progress().to_bytes().expect("review progress"),
        solved_bytes
    );
    assert!(solved.is_current_solved());
}

#[test]
fn review_main_line_and_variation_selection_are_deterministic() {
    let mut app = puzzle_state(PUZZLES);
    add_reviews(&mut app);
    app.dispatch(Action::ToggleWorkspace);
    let root = review_index(&app, "n0");
    let n1 = review_index(&app, "n1");
    let n2 = review_index(&app, "n2");
    let variation = review_index(&app, "v2");

    let review = app.review_state().expect("review state");
    assert_eq!(review.selected_node(), root);
    assert_eq!(review.main_line_ply(), 0);
    assert!(!review.can_previous());
    assert!(review.can_next(app.active_review_game().expect("game").game()));

    let root_board = app.board().clone();
    app.dispatch(Action::ReviewPrevious);
    assert_eq!(app.board(), &root_board);
    app.dispatch(Action::ReviewNext);
    assert_eq!(app.selected_analysis_node(), Some(n1));
    assert_eq!(app.review_state().expect("review").main_line_ply(), 1);
    assert_eq!(
        app.review_state().expect("review").analysis_focus(),
        Some(n1)
    );

    app.dispatch(Action::AnalysisNextPage);
    assert_eq!(app.analysis_page(), 1);
    assert_eq!(app.review_state().expect("review").analysis_focus(), None);
    app.dispatch(Action::SelectAnalysisNode(variation));
    assert_eq!(
        app.analysis_page(),
        1,
        "selection keeps explicit page state"
    );
    assert_eq!(app.selected_analysis_node(), Some(variation));
    assert_eq!(
        app.review_state().expect("review").main_line_ply(),
        1,
        "variation navigation anchors to its nearest main-line ancestor"
    );
    let variation_fen = &app
        .active_review_game()
        .expect("game")
        .game()
        .analysis
        .node(variation)
        .expect("variation")
        .fen;
    assert_eq!(
        app.board(),
        &Board::from_fen(variation_fen).expect("validated variation FEN")
    );

    app.dispatch(Action::ReviewNext);
    assert_eq!(app.selected_analysis_node(), Some(n2));
    while app
        .review_state()
        .expect("review")
        .can_next(app.active_review_game().expect("game").game())
    {
        app.dispatch(Action::ReviewNext);
    }
    let end = app.clone();
    app.dispatch(Action::ReviewNext);
    assert_eq!(app, end);
}

#[test]
fn review_free_board_is_reversible_and_tracks_authored_selection_changes() {
    let mut app = puzzle_state(PUZZLES);
    add_reviews(&mut app);
    app.dispatch(Action::ToggleWorkspace);
    let root = app.board().clone();
    let v1 = review_index(&app, "v1");
    let n2 = review_index(&app, "n2");

    app.dispatch(Action::ReviewToggleFree);
    assert!(app.review_state().expect("review").free_board_enabled());
    app.dispatch(Action::TapSquare(square("e2")));
    app.dispatch(Action::TapSquare(square("e4")));
    assert_ne!(app.board(), &root);
    assert_eq!(
        app.review_state().expect("review").authored_board(),
        &root,
        "scratch moves do not mutate authored position"
    );

    app.dispatch(Action::ReviewReset);
    assert_eq!(app.board(), &root);

    app.dispatch(Action::SelectAnalysisNode(v1));
    assert!(app.review_state().expect("review").free_board_enabled());
    let variation = app.review_state().expect("review").authored_board().clone();
    assert_eq!(app.board(), &variation);

    app.dispatch(Action::TapSquare(square("g1")));
    app.dispatch(Action::TapSquare(square("f3")));
    assert_ne!(app.board(), &variation);
    app.dispatch(Action::ReviewNext);
    assert_eq!(app.selected_analysis_node(), Some(n2));
    assert!(app.review_state().expect("review").free_board_enabled());
    assert_eq!(
        app.board(),
        app.review_state().expect("review").authored_board(),
        "main-line navigation resets scratch to the newly selected authored FEN"
    );

    app.dispatch(Action::TapSquare(square("g1")));
    app.dispatch(Action::TapSquare(square("f3")));
    app.dispatch(Action::ReviewToggleFree);
    assert!(!app.review_state().expect("review").free_board_enabled());
    assert_eq!(
        app.board(),
        app.review_state().expect("review").authored_board()
    );
}

#[test]
fn review_game_switch_resets_navigation_preserves_locked_orientation_and_source_data() {
    let mut app = puzzle_state(PUZZLES);
    let standard = review_entry("games-standard.json", REVIEW);
    let custom = review_entry("games-custom.json", CUSTOM_REVIEW);
    let custom_source = custom.game().clone();
    let custom_key = custom.key().clone();
    app.set_review_games(vec![standard, custom]);
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ReviewToggleOrientationLock);
    assert!(!app.flipped());
    app.dispatch(Action::ReviewNext);
    app.dispatch(Action::AnalysisNextPage);
    app.dispatch(Action::ReviewToggleFree);

    app.dispatch(Action::OpenReviewGamePicker);
    assert!(app.review_state().expect("review").game_picker_open());
    assert_eq!(
        app.dispatch(Action::SelectReviewGame(1)),
        vec![Effect::ReviewGameRequested(custom_key.clone())]
    );
    assert!(app.review_state().expect("review").game_picker_open());
    assert!(app
        .dispatch(Action::ActivateReviewGame(
            custom_key,
            custom_source.clone(),
        ))
        .is_empty());

    assert_eq!(
        app.active_review_game()
            .expect("active game")
            .key()
            .collection_id(),
        "games-custom.json"
    );
    let review = app.review_state().expect("review");
    assert_eq!(
        review.selected_node(),
        app.active_review_game()
            .expect("game")
            .game()
            .analysis
            .root_index()
    );
    assert_eq!(review.main_line_ply(), 0);
    assert_eq!(review.analysis_page(), 0);
    assert!(!review.free_board_enabled());
    assert!(!review.game_picker_open());
    assert!(review.orientation_locked());
    assert!(!review.flipped(), "locked orientation survives game switch");
    assert_eq!(
        app.active_review_game().expect("game").game(),
        &custom_source,
        "review actions never mutate source game data"
    );
}

#[test]
fn review_picker_and_settings_modals_block_review_navigation_and_workspace_switching() {
    let mut app = puzzle_state(RICH);
    add_reviews(&mut app);
    app.dispatch(Action::OpenAnalysis);
    let puzzle_node = puzzle_analysis_index(&app, "n1");
    app.dispatch(Action::SelectAnalysisNode(puzzle_node));
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ReviewToggleFree);

    app.dispatch(Action::OpenReviewGamePicker);
    let picker_open = app.clone();
    app.dispatch(Action::ReviewNext);
    app.dispatch(Action::TapSquare(square("e2")));
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app, picker_open);

    app.dispatch(Action::CloseReviewGamePicker);
    app.dispatch(Action::OpenSettings);
    assert!(app.settings_open());
    let settings_open = app.clone();
    app.dispatch(Action::ReviewNext);
    app.dispatch(Action::ReviewReset);
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app, settings_open);

    app.dispatch(Action::CloseSettings);
    app.dispatch(Action::TapSquare(square("e2")));
    assert_eq!(
        app.board().selected(),
        Some(square("e2")),
        "open puzzle analysis is preserved but does not block review FREE taps"
    );
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.workspace(), Workspace::Puzzles);
    assert_eq!(app.selected_analysis_node(), Some(puzzle_node));
}
