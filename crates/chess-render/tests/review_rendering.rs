use chess_core::{
    algebraic_to_square, parse_puzzle_file, parse_review_file, Action, ActiveCollection, AppState,
    Progress, ReviewGameEntry, Workspace,
};
use chess_render::{render, AnalysisMoveChipSource, DisplayMetrics, HitTarget};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const STANDARD: &[u8] = include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
const CUSTOM: &[u8] = include_bytes!("../../../tests/fixtures/game-review/valid-custom-fen.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn puzzle_state() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(PUZZLES).expect("puzzle fixture"),
        ),
        Progress::new(),
    )
}

fn review_entries(collection_id: &str, bytes: &[u8]) -> Vec<ReviewGameEntry> {
    parse_review_file(bytes)
        .expect("review fixture")
        .games
        .into_iter()
        .map(|game| ReviewGameEntry::new(collection_id, game))
        .collect()
}

fn review_state(bytes: &[u8], collection_id: &str) -> AppState {
    let mut app = puzzle_state();
    app.set_review_games(review_entries(collection_id, bytes));
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.workspace(), Workspace::Review);
    app
}

fn two_game_state() -> AppState {
    let mut app = puzzle_state();
    let mut games = review_entries("games-standard.json", STANDARD);
    games.extend(review_entries("games-custom.json", CUSTOM));
    app.set_review_games(games);
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.workspace(), Workspace::Review);
    app
}

fn node(app: &AppState, id: &str) -> chess_core::AnalysisNodeIndex {
    app.active_review_game()
        .expect("review game")
        .game()
        .analysis
        .node_index(id)
        .expect("review node")
}

fn hash(app: &AppState) -> u64 {
    let output = render(app, SCRIBE).expect("review render");
    if let Ok(directory) = std::env::var("UI_SNAPSHOT_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            format!("{directory}/{}.pgm", output.frame.checksum64()),
            output.frame.to_pgm(),
        )
        .unwrap();
    }
    output.frame.checksum64()
}

fn long_review() -> AppState {
    let mut value: serde_json::Value = serde_json::from_slice(STANDARD).unwrap();
    value["games"][0]["analysis"]["nodes"][0]["content"][0]["text"] =
        "Long annotated context before the moves keeps deterministic overflow pages readable. "
            .repeat(28)
            .into();
    review_state(&serde_json::to_vec(&value).unwrap(), "games-long.json")
}

#[test]
fn review_layout_keeps_touch_targets_and_replaces_goto_with_two_main_line_controls() {
    let app = review_state(STANDARD, "games-standard.json");
    let output = render(&app, SCRIBE).unwrap();
    let layout = output.layout;
    let minimum = layout.minimum_touch_px();

    assert_eq!(layout.workspace.x, layout.refresh.right());
    assert!(layout.workspace.width >= minimum);
    assert_eq!(layout.goto.width, 0);
    for rect in [layout.previous, layout.next] {
        assert!(rect.width >= minimum);
        assert!(rect.height >= minimum);
    }

    let visible = layout
        .toolbar_targets
        .into_iter()
        .filter(|target| target.rect.width > 0)
        .collect::<Vec<_>>();
    assert_eq!(visible.len(), 4);
    assert_eq!(
        visible
            .iter()
            .map(|target| target.target)
            .collect::<Vec<_>>(),
        vec![
            HitTarget::ToggleMode,
            HitTarget::Reset,
            HitTarget::ToggleOrientationLock,
            HitTarget::Flip,
        ]
    );
    assert!(visible
        .iter()
        .all(|target| target.rect.width >= minimum && target.rect.height >= minimum));
}

#[test]
fn review_movetext_is_always_visible_and_focuses_the_selected_move_page() {
    let mut app = long_review();
    let root = render(&app, SCRIBE).unwrap();
    assert!(root.analysis.is_some(), "review analysis is always visible");

    for _ in 0..7 {
        app.dispatch(Action::ReviewNext);
    }
    let selected = app.selected_analysis_node().expect("selected final move");
    let focused = render(&app, SCRIBE).unwrap();
    let panel = focused.analysis.as_ref().expect("review analysis");
    assert!(
        panel.page > 0,
        "long prefix should push the final move off page one"
    );
    assert!(panel.move_chips.iter().any(|chip| {
        chip.source == AnalysisMoveChipSource::TreeMove && chip.node == selected && chip.selected
    }));

    app.dispatch(Action::AnalysisPreviousPage);
    let manually_browsed = render(&app, SCRIBE).unwrap();
    assert_eq!(
        manually_browsed.analysis.as_ref().unwrap().page,
        0,
        "explicit page browsing clears automatic focus and uses requested pages"
    );
}

#[test]
fn small_review_board_preserves_touch_geometry_and_uses_larger_analysis_text() {
    let mut app = review_state(STANDARD, "games-standard.json");
    let standard = render(&app, SCRIBE).unwrap();
    let standard_chip = standard.analysis.as_ref().unwrap().move_chips[0].rect;

    app.dispatch(Action::ToggleBoardSizeSetting);
    let small = render(&app, SCRIBE).unwrap();
    let small_chip = small.analysis.as_ref().unwrap().move_chips[0].rect;

    assert!(small.layout.board.width < standard.layout.board.width);
    assert!(small.layout.status.height > standard.layout.status.height);
    assert!(small_chip.height > standard_chip.height);
    for target in small.layout.toolbar_targets {
        if target.rect.width > 0 {
            assert!(target.rect.width >= small.layout.minimum_touch_px());
            assert!(target.rect.height >= small.layout.minimum_touch_px());
        }
    }
    assert!(small.layout.previous.width >= small.layout.minimum_touch_px());
    assert!(small.layout.next.width >= small.layout.minimum_touch_px());
}

#[test]
fn review_free_board_changes_board_pixels_without_losing_authored_move_selection() {
    let mut app = review_state(STANDARD, "games-standard.json");
    app.dispatch(Action::ReviewNext);
    let selected = app.selected_analysis_node().unwrap();
    app.dispatch(Action::ReviewToggleFree);
    app.dispatch(Action::TapSquare(square("g1")));
    app.dispatch(Action::TapSquare(square("f3")));

    let output = render(&app, SCRIBE).unwrap();
    assert!(output
        .analysis
        .as_ref()
        .unwrap()
        .move_chips
        .iter()
        .any(|chip| {
            chip.source == AnalysisMoveChipSource::TreeMove
                && chip.node == selected
                && chip.selected
        }));
    assert_ne!(
        app.board(),
        app.review_state().unwrap().authored_board(),
        "scratch board should visibly diverge from the authored FEN"
    );
}

#[test]
fn review_workspace_visual_states_match_reviewed_gray8_snapshots() {
    let mut actual = Vec::new();

    let root = review_state(STANDARD, "games-standard.json");
    actual.push(("review-root-standard", hash(&root)));

    let mut middle = root.clone();
    for _ in 0..3 {
        middle.dispatch(Action::ReviewNext);
    }
    actual.push(("review-middle-main", hash(&middle)));

    let mut final_main = root.clone();
    for _ in 0..7 {
        final_main.dispatch(Action::ReviewNext);
    }
    actual.push(("review-final-main", hash(&final_main)));

    let mut variation = root.clone();
    let v1 = node(&variation, "v1");
    variation.dispatch(Action::SelectAnalysisNode(v1));
    actual.push(("review-selected-variation", hash(&variation)));

    let first = long_review();
    let first_output = render(&first, SCRIBE).unwrap();
    let page_count = first_output.analysis.as_ref().unwrap().page_count;
    assert!(page_count >= 3);
    actual.push(("review-long-first", first_output.frame.checksum64()));

    let mut long_middle = first.clone();
    for _ in 0..page_count / 2 {
        long_middle.dispatch(Action::AnalysisNextPage);
    }
    actual.push(("review-long-middle", hash(&long_middle)));

    let mut long_last = first;
    for _ in 0..page_count.saturating_sub(1) {
        long_last.dispatch(Action::AnalysisNextPage);
    }
    actual.push(("review-long-last", hash(&long_last)));

    let black = review_state(CUSTOM, "games-custom.json");
    assert!(
        black.flipped(),
        "black-to-move review starts oriented for Black"
    );
    actual.push(("review-black-to-move", hash(&black)));

    let mut unicode_value: serde_json::Value = serde_json::from_slice(STANDARD).unwrap();
    unicode_value["games"][0]["white"] =
        "José Raúl Capablanca — København — очень длинное имя".into();
    unicode_value["games"][0]["black"] = "Александр Алехин — München — très longue identité".into();
    let unicode = review_state(
        &serde_json::to_vec(&unicode_value).unwrap(),
        "games-unicode.json",
    );
    actual.push(("review-unicode-header", hash(&unicode)));

    let mut free = root.clone();
    free.dispatch(Action::ReviewNext);
    free.dispatch(Action::ReviewToggleFree);
    free.dispatch(Action::TapSquare(square("g1")));
    free.dispatch(Action::TapSquare(square("f3")));
    actual.push(("review-free-scratch", hash(&free)));

    let mut flipped = root.clone();
    flipped.dispatch(Action::ReviewFlip);
    actual.push(("review-flipped", hash(&flipped)));

    let mut small = root.clone();
    small.dispatch(Action::ToggleBoardSizeSetting);
    actual.push(("review-small-board", hash(&small)));

    let mut picker = two_game_state();
    picker.dispatch(Action::OpenReviewGamePicker);
    let picker_output = render(&picker, SCRIBE).expect("picker render");
    let close = picker_output.layout.review_game_close;
    assert_eq!(
        picker_output.hit_test_app(
            close.x + close.width / 2,
            close.y + close.height / 2,
            &picker
        ),
        Some(HitTarget::CloseReviewGamePicker)
    );
    let row = picker_output.layout.review_game_rows[0];
    assert_eq!(
        picker_output.hit_test_app(row.x + row.width / 2, row.y + row.height / 2, &picker),
        Some(HitTarget::ReviewGame(0))
    );
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::write(
        root.join("review-games-dialog.pgm"),
        picker_output.frame.to_pgm(),
    )
    .unwrap();
    actual.push(("review-game-picker", picker_output.frame.checksum64()));

    let mut no_games = puzzle_state();
    no_games.dispatch(Action::ToggleWorkspace);
    assert_eq!(no_games.workspace(), Workspace::Puzzles);
    assert!(no_games.transient_message().is_some());
    actual.push(("review-no-games", hash(&no_games)));

    const EXPECTED: &[(&str, u64)] = &[
        ("review-root-standard", 11_072_394_659_977_528_714),
        ("review-middle-main", 2_014_352_177_180_291_752),
        ("review-final-main", 13_880_241_759_628_312_160),
        ("review-selected-variation", 17_973_672_245_252_132_704),
        ("review-long-first", 16_395_587_122_926_988_907),
        ("review-long-middle", 13_505_780_319_366_563_043),
        ("review-long-last", 3_779_354_846_187_004_797),
        ("review-black-to-move", 395_374_924_477_436_930),
        ("review-unicode-header", 997_277_629_216_496_538),
        ("review-free-scratch", 9_334_775_776_239_098_211),
        ("review-flipped", 15_807_205_748_985_471_160),
        ("review-small-board", 16_894_878_170_182_868_405),
        ("review-game-picker", 16_287_243_912_448_463_606),
        ("review-no-games", 11_567_526_096_340_990_005),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
