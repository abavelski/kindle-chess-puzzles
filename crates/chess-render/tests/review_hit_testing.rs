use std::collections::HashSet;

use chess_core::{
    parse_puzzle_file, parse_review_file, Action, ActiveCollection, AnalysisNodeIndex, AppState,
    Board, Progress, ReviewFileError, ReviewGameEntry, Workspace,
};
use chess_render::{
    render, AnalysisMoveChipSource, DisplayMetrics, HitTarget, Rect, RenderOutput,
};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const REVIEW: &[u8] = include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn puzzle_state() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(PUZZLES).expect("puzzle fixture"),
        ),
        Progress::new(),
    )
}

fn review_game() -> chess_core::ReviewGame {
    parse_review_file(REVIEW)
        .expect("review fixture")
        .games
        .into_iter()
        .next()
        .expect("review game")
}

fn review_state() -> AppState {
    let mut app = puzzle_state();
    app.set_review_games(vec![ReviewGameEntry::new(
        "games-standard.json",
        review_game(),
    )]);
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.workspace(), Workspace::Review);
    app
}

fn long_review_state() -> AppState {
    let mut value: serde_json::Value = serde_json::from_slice(REVIEW).expect("review json");
    value["games"][0]["analysis"]["nodes"][0]["content"][0]["text"] =
        ("Plain SAN/UCI-looking prose e4 c5 g1f3 stays inert. ".repeat(32)).into();
    let collection =
        parse_review_file(&serde_json::to_vec(&value).expect("review json")).expect("long review");
    let mut app = puzzle_state();
    app.set_review_games(vec![ReviewGameEntry::new(
        "games-long.json",
        collection.games[0].clone(),
    )]);
    app.dispatch(Action::ToggleWorkspace);
    app
}

fn center(rect: Rect) -> (u32, u32) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}

fn assert_rect_target(
    output: &RenderOutput,
    app: &AppState,
    rect: Rect,
    expected: HitTarget,
) {
    assert!(rect.width > 0 && rect.height > 0);
    let points = [
        center(rect),
        (rect.x, rect.y),
        (rect.right() - 1, rect.y),
        (rect.x, rect.bottom() - 1),
        (rect.right() - 1, rect.bottom() - 1),
    ];
    for (x, y) in points {
        assert_eq!(
            output.hit_test_app(x, y, app),
            Some(expected),
            "unexpected target at ({x}, {y}) inside {rect:?}"
        );
    }
}

#[test]
fn review_controls_map_to_review_actions_while_puzzle_controls_keep_their_actions() {
    let review = review_state();
    let output = render(&review, SCRIBE).expect("review render");
    let layout = output.layout;

    let review_controls = [
        (
            layout.workspace,
            HitTarget::ToggleWorkspace,
            Some(Action::ToggleWorkspace),
        ),
        (
            layout.collection_button,
            HitTarget::OpenReviewGamePicker,
            Some(Action::OpenReviewGamePicker),
        ),
        (
            layout.toolbar_targets[0].rect,
            HitTarget::ReviewToggleFree,
            Some(Action::ReviewToggleFree),
        ),
        (
            layout.toolbar_targets[1].rect,
            HitTarget::ReviewReset,
            Some(Action::ReviewReset),
        ),
        (
            layout.toolbar_targets[2].rect,
            HitTarget::ReviewToggleOrientationLock,
            Some(Action::ReviewToggleOrientationLock),
        ),
        (
            layout.toolbar_targets[3].rect,
            HitTarget::ReviewFlip,
            Some(Action::ReviewFlip),
        ),
        (
            layout.previous,
            HitTarget::ReviewPrevious,
            Some(Action::ReviewPrevious),
        ),
        (
            layout.next,
            HitTarget::ReviewNext,
            Some(Action::ReviewNext),
        ),
    ];
    for (rect, target, action) in review_controls {
        assert_rect_target(&output, &review, rect, target);
        assert_eq!(target.into_action(), action);
    }

    let puzzle = puzzle_state();
    let output = render(&puzzle, SCRIBE).expect("puzzle render");
    let layout = output.layout;
    assert_rect_target(
        &output,
        &puzzle,
        layout.workspace,
        HitTarget::ToggleWorkspace,
    );
    assert_rect_target(
        &output,
        &puzzle,
        layout.toolbar_targets[1].rect,
        HitTarget::ToggleMode,
    );
    assert_rect_target(
        &output,
        &puzzle,
        layout.toolbar_targets[3].rect,
        HitTarget::ToggleOrientationLock,
    );
    assert_rect_target(&output, &puzzle, layout.previous, HitTarget::Previous);
    assert_rect_target(
        &output,
        &puzzle,
        layout.goto,
        HitTarget::OpenPuzzleGoto,
    );
    assert_rect_target(&output, &puzzle, layout.next, HitTarget::Next);
    assert_eq!(
        HitTarget::Previous.into_action(),
        Some(Action::PreviousPuzzle)
    );
    assert_eq!(HitTarget::Next.into_action(), Some(Action::NextPuzzle));
}

#[test]
fn review_board_hit_testing_is_read_only_until_free_and_respects_flip_and_board_size() {
    let mut app = review_state();
    let output = render(&app, SCRIBE).expect("review render");
    let point = center(output.layout.square_rect(0));
    assert_eq!(output.hit_test_app(point.0, point.1, &app), None);

    app.dispatch(Action::ReviewToggleFree);
    let standard = render(&app, SCRIBE).expect("free review render");
    let point = center(standard.layout.square_rect(0));
    assert_eq!(
        standard.hit_test_app(point.0, point.1, &app),
        Some(HitTarget::Square(0))
    );
    assert_eq!(
        standard
            .hit_test_app(point.0, point.1, &app)
            .and_then(HitTarget::into_action),
        Some(Action::TapSquare(0))
    );

    app.dispatch(Action::ReviewFlip);
    let flipped = render(&app, SCRIBE).expect("flipped review render");
    let point = center(flipped.layout.square_rect(0));
    assert_eq!(
        flipped.hit_test_app(point.0, point.1, &app),
        Some(HitTarget::Square(63))
    );

    app.dispatch(Action::ToggleBoardSizeSetting);
    let small = render(&app, SCRIBE).expect("small review render");
    assert!(small.layout.board.width < flipped.layout.board.width);
    let point = center(small.layout.square_rect(0));
    assert_eq!(
        small.hit_test_app(point.0, point.1, &app),
        Some(HitTarget::Square(63))
    );
}

#[test]
fn review_movetext_reuses_move_and_page_hit_targets_without_linking_plain_prose() {
    let mut app = long_review_state();
    let mut tree_nodes = HashSet::<AnalysisNodeIndex>::new();
    let mut tree_chips = 0;
    let mut inline_chips = 0;
    let mut pages = 0;

    loop {
        let output = render(&app, SCRIBE).expect("review render");
        let panel = output.analysis.as_ref().expect("review analysis");
        pages += 1;

        if let Some(previous) = panel.previous_page {
            assert_rect_target(
                &output,
                &app,
                previous,
                HitTarget::AnalysisPreviousPage,
            );
            assert_eq!(
                HitTarget::AnalysisPreviousPage.into_action(),
                Some(Action::AnalysisPreviousPage)
            );
        }

        for chip in &panel.move_chips {
            assert_rect_target(
                &output,
                &app,
                chip.rect,
                HitTarget::AnalysisMove(chip.node),
            );
            assert_eq!(
                HitTarget::AnalysisMove(chip.node).into_action(),
                Some(Action::SelectAnalysisNode(chip.node))
            );

            let node = app
                .active_review_game()
                .expect("review game")
                .game()
                .analysis
                .node(chip.node)
                .expect("chip node");
            let expected = Board::from_fen(&node.fen).expect("stored review FEN");
            let mut tapped = app.clone();
            tapped.dispatch(Action::SelectAnalysisNode(chip.node));
            assert_eq!(tapped.board(), &expected);

            match chip.source {
                AnalysisMoveChipSource::TreeMove => {
                    tree_chips += 1;
                    tree_nodes.insert(chip.node);
                }
                AnalysisMoveChipSource::InlineReference => inline_chips += 1,
            }
        }

        let Some(next) = panel.next_page else {
            break;
        };
        assert_rect_target(&output, &app, next, HitTarget::AnalysisNextPage);
        assert_eq!(
            HitTarget::AnalysisNextPage.into_action(),
            Some(Action::AnalysisNextPage)
        );
        app.dispatch(Action::AnalysisNextPage);
        assert!(pages < 32, "analysis pagination must terminate");
    }

    let analysis = &app.active_review_game().expect("review game").game().analysis;
    assert!(pages > 1);
    assert_eq!(
        tree_chips,
        analysis.nodes().len() - 1,
        "every authored non-root tree move appears exactly once"
    );
    assert_eq!(tree_nodes.len(), analysis.nodes().len() - 1);
    assert_eq!(
        inline_chips, 1,
        "only the explicit move_ref is interactive; plain e4/c5/g1f3 prose is inert"
    );
    for id in ["n1", "n7", "v1", "v2"] {
        assert!(tree_nodes.contains(&analysis.node_index(id).expect("fixture node")));
    }
}

#[test]
fn games_picker_paginates_stable_game_keys_blocks_invalid_rows_and_preserves_progress() {
    let game = review_game();
    let games = (0..7)
        .map(|index| ReviewGameEntry::new(format!("games-{index}.json"), game.clone()))
        .collect::<Vec<_>>();
    let mut app = puzzle_state();
    app.set_review_games(games);
    app.set_review_file_errors(vec![ReviewFileError::new(
        "games-broken.json",
        "Invalid review file: expected JSON",
    )]);
    app.dispatch(Action::ToggleWorkspace);

    let first = &app.review_games()[0];
    assert_eq!(first.key().collection_id(), "games-0.json");
    assert_eq!(first.key().game_id(), "unicode-castling-sample");
    assert_eq!(
        first.label(),
        "José Raúl Capablanca - Александр Алехин\nТестовый турнир — København / 1927.09.16  1/2-1/2"
    );
    assert_eq!(app.review_picker_entry_count(), 8);
    let progress_before = app.progress().to_bytes().expect("progress bytes");

    let output = render(&app, SCRIBE).expect("review render");
    let point = center(output.layout.collection_button);
    let open = output
        .hit_test_app(point.0, point.1, &app)
        .expect("GAMES hit");
    assert_eq!(open, HitTarget::OpenReviewGamePicker);
    app.dispatch(open.into_action().expect("GAMES action"));

    let page_one = render(&app, SCRIBE).expect("picker page one");
    for index in 0..6 {
        assert_rect_target(
            &page_one,
            &app,
            page_one.layout.collection_rows[index],
            HitTarget::ReviewGame(index),
        );
    }
    assert_rect_target(
        &page_one,
        &app,
        page_one.layout.collection_page_next,
        HitTarget::ReviewGamePickerNextPage,
    );
    let board_point = center(page_one.layout.square_rect(0));
    assert_eq!(
        page_one.hit_test_app(board_point.0, board_point.1, &app),
        None,
        "picker modal blocks board hits"
    );
    app.dispatch(Action::ReviewGamePickerNextPage);

    let page_two = render(&app, SCRIBE).expect("picker page two");
    assert_rect_target(
        &page_two,
        &app,
        page_two.layout.collection_rows[0],
        HitTarget::ReviewGame(6),
    );
    let invalid = center(page_two.layout.collection_rows[1]);
    assert_eq!(
        page_two.hit_test_app(invalid.0, invalid.1, &app),
        None,
        "invalid review-file rows are visible diagnostics, not activatable games"
    );
    assert_rect_target(
        &page_two,
        &app,
        page_two.layout.collection_page_previous,
        HitTarget::ReviewGamePickerPreviousPage,
    );
    assert_rect_target(
        &page_two,
        &app,
        page_two.layout.collection_close,
        HitTarget::CloseReviewGamePicker,
    );

    let invalid_before = app.clone();
    app.dispatch(Action::SelectReviewGame(7));
    assert_eq!(app, invalid_before, "invalid picker index must be inert");

    let select = HitTarget::ReviewGame(6)
        .into_action()
        .expect("review-game action");
    app.dispatch(select);
    let review = app.review_state().expect("review state");
    assert!(!review.game_picker_open());
    assert_eq!(review.main_line_ply(), 0);
    assert_eq!(
        review.selected_node(),
        app.active_review_game()
            .expect("active review game")
            .game()
            .analysis
            .root_index()
    );
    assert_eq!(
        app.active_review_game()
            .expect("active review game")
            .key()
            .collection_id(),
        "games-6.json"
    );
    assert_eq!(
        app.progress().to_bytes().expect("progress bytes"),
        progress_before,
        "review picker selection must not touch puzzle progress"
    );
}
