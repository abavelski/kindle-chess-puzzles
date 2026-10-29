use chess_core::{
    parse_puzzle_file, parse_review_file, AnalysisRole, AnalysisTextSpan, ReviewGameKey,
    ReviewResult, MAX_REVIEW_FILE_BYTES,
};
use serde_json::json;
use std::collections::HashSet;

const VALID_STANDARD: &[u8] =
    include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
const VALID_CUSTOM_FEN: &[u8] =
    include_bytes!("../../../tests/fixtures/game-review/valid-custom-fen.json");

const INVALID_FIXTURES: &[(&str, &[u8])] = &[
    (
        "duplicate game id",
        include_bytes!("../../../tests/fixtures/game-review/invalid-duplicate-game-id.json"),
    ),
    (
        "missing fen",
        include_bytes!("../../../tests/fixtures/game-review/invalid-missing-fen.json"),
    ),
    (
        "root fen mismatch",
        include_bytes!("../../../tests/fixtures/game-review/invalid-root-fen-mismatch.json"),
    ),
    (
        "duplicate node id",
        include_bytes!("../../../tests/fixtures/game-review/invalid-duplicate-node-id.json"),
    ),
    (
        "missing child",
        include_bytes!("../../../tests/fixtures/game-review/invalid-missing-child.json"),
    ),
    (
        "cycle",
        include_bytes!("../../../tests/fixtures/game-review/invalid-cycle.json"),
    ),
    (
        "disconnected",
        include_bytes!("../../../tests/fixtures/game-review/invalid-disconnected.json"),
    ),
    (
        "invalid uci",
        include_bytes!("../../../tests/fixtures/game-review/invalid-uci.json"),
    ),
    (
        "invalid node fen",
        include_bytes!("../../../tests/fixtures/game-review/invalid-node-fen.json"),
    ),
    (
        "empty main line",
        include_bytes!("../../../tests/fixtures/game-review/invalid-empty-main-line.json"),
    ),
    (
        "dangling move ref",
        include_bytes!("../../../tests/fixtures/game-review/invalid-move-ref-dangling.json"),
    ),
    (
        "solution field",
        include_bytes!("../../../tests/fixtures/game-review/invalid-solution-field.json"),
    ),
];

#[test]
fn task_41_standard_fixture_parses_into_typed_review_model() {
    let collection = parse_review_file(VALID_STANDARD).expect("standard review fixture");
    assert_eq!(collection.version(), 1);
    assert_eq!(collection.title.as_deref(), Some("Annotated classics"));
    assert_eq!(
        collection.source.as_deref(),
        Some("Task 41 contract fixture")
    );
    assert_eq!(collection.games.len(), 1);

    let game = &collection.games[0];
    assert_eq!(game.id, "unicode-castling-sample");
    assert_eq!(game.metadata.white, "José Raúl Capablanca");
    assert_eq!(game.metadata.black, "Александр Алехин");
    assert_eq!(game.metadata.result, ReviewResult::Draw);
    assert_eq!(game.metadata.result.as_str(), "1/2-1/2");
    assert_eq!(game.metadata.event, "Тестовый турнир — København");
    assert_eq!(game.metadata.site, "København, DK");
    assert_eq!(game.metadata.date, "1927.09.16");
    assert_eq!(game.metadata.round, "1");
    assert_eq!(
        game.metadata.source.as_deref(),
        Some("synthetic contract example")
    );

    let analysis = &game.analysis;
    assert_eq!(analysis.root().id, "n0");
    assert_eq!(analysis.main_line_nodes().len(), 8);
    assert_eq!(
        analysis
            .node(analysis.main_line_node_at_ply(0).expect("root ply"))
            .expect("root"),
        analysis.root()
    );
    assert_eq!(
        analysis
            .node(analysis.main_line_node_at_ply(7).expect("seventh ply"))
            .expect("last main node")
            .id,
        "n7"
    );
    assert_eq!(analysis.main_line_node_at_ply(8), None);

    let n1 = analysis.node_index("n1").expect("n1");
    let v1 = analysis.node_index("v1").expect("v1");
    let v2 = analysis.node_index("v2").expect("v2");
    assert_eq!(analysis.main_line_ply(n1), Some(1));
    assert!(analysis.is_main_line(n1));
    assert_eq!(analysis.main_line_ply(v1), None);
    assert!(!analysis.is_main_line(v1));
    assert_eq!(analysis.nearest_main_line_ancestor(v1), Some(n1));
    assert_eq!(analysis.nearest_main_line_ancestor(v2), Some(n1));

    match &analysis.root().content[1] {
        AnalysisTextSpan::MoveRef { node, label } => {
            assert_eq!(analysis.node(*node).expect("move ref target").id, "n7");
            assert_eq!(label.as_deref(), Some("4.O-O"));
        }
        AnalysisTextSpan::Text(_) => panic!("expected root move_ref"),
    }
}

#[test]
fn task_41_custom_fen_fixture_preserves_black_to_move_and_variation_roles() {
    let collection = parse_review_file(VALID_CUSTOM_FEN).expect("custom review fixture");
    let game = &collection.games[0];
    assert_eq!(game.id, "setup-black-promotion");
    assert_eq!(game.fen.split_ascii_whitespace().nth(1), Some("b"));
    assert_eq!(game.metadata.result, ReviewResult::Ongoing);

    let analysis = &game.analysis;
    let main = analysis.main_line_node_at_ply(1).expect("main promotion");
    let main_node = analysis.node(main).expect("main node");
    assert_eq!(main_node.movement.as_ref().expect("move").uci, "h2h1q");
    assert_eq!(main_node.role, Some(AnalysisRole::Main));

    let variation = analysis.node_index("v1").expect("underpromotion");
    assert_eq!(
        analysis.node(variation).expect("variation").role,
        Some(AnalysisRole::Alternative)
    );
    assert_eq!(
        analysis.nearest_main_line_ancestor(variation),
        Some(analysis.root_index())
    );
}

#[test]
fn every_task_41_invalid_fixture_is_rejected() {
    for (label, fixture) in INVALID_FIXTURES {
        let error = parse_review_file(fixture).expect_err(label);
        assert!(!error.is_empty(), "{label} should provide context");
    }
}

#[test]
fn solution_field_is_rejected_explicitly() {
    let fixture = include_bytes!("../../../tests/fixtures/game-review/invalid-solution-field.json");
    let error = parse_review_file(fixture).expect_err("solution is puzzle-only");
    assert!(
        error.contains("must not contain a solution field"),
        "{error}"
    );
}

#[test]
fn puzzle_projection_mismatch_remains_puzzle_only_and_keeps_existing_error() {
    let review_value: serde_json::Value =
        serde_json::from_slice(VALID_STANDARD).expect("review JSON");
    let game = &review_value["games"][0];
    let puzzle_value = json!({
        "version": 1,
        "puzzles": [{
            "id": "same-tree",
            "fen": game["fen"].clone(),
            "solution": ["e2e4"],
            "analysis": game["analysis"].clone()
        }]
    });

    let error = parse_puzzle_file(&serde_json::to_vec(&puzzle_value).expect("puzzle JSON"))
        .expect_err("short legacy solution must not match the seven-ply analysis");
    assert!(
        error.contains("analysis main-path UCI sequence must exactly match legacy solution."),
        "{error}"
    );

    let review = parse_review_file(VALID_STANDARD)
        .expect("the same analysis tree is valid without a synthetic review solution");
    assert_eq!(review.games[0].analysis.main_line_nodes().len(), 8);
}

#[test]
fn review_game_key_is_stable_and_distinguishes_collection_identity() {
    let collection = parse_review_file(VALID_STANDARD).expect("standard review fixture");
    let game = &collection.games[0];

    let first = game.key("games-classics.json");
    let same = ReviewGameKey::new("games-classics.json", game.id.clone());
    let other_file = game.key("games.json");

    assert_eq!(first, same);
    assert_ne!(first, other_file);
    assert_eq!(first.collection_id(), "games-classics.json");
    assert_eq!(first.game_id(), game.id.as_str());

    let keys = HashSet::from([first.clone(), same, other_file]);
    assert_eq!(keys.len(), 2);
}

#[test]
fn review_file_cap_matches_the_frozen_eight_mib_contract() {
    let oversized = vec![b' '; MAX_REVIEW_FILE_BYTES + 1];
    assert_eq!(
        parse_review_file(&oversized).expect_err("oversized review file"),
        "Review file must be at most 8 MiB (8,388,608 bytes)."
    );
}
