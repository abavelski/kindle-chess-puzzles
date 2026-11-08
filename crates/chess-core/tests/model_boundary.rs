use chess_model::analysis::parse_analysis_tree;

#[test]
fn generic_review_tree_needs_no_puzzle_solution() {
    let document: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap();
    let game = &document["games"][0];
    let tree = parse_analysis_tree(
        &game["analysis"],
        game["fen"].as_str().unwrap(),
        "game fen",
        "test",
    )
    .unwrap();
    assert!(tree.main_line_nodes().len() > 1);
    // The compatibility re-export must be the same type, without conversion/copying.
    let _: chess_core::AnalysisTree = tree;
}

#[test]
fn generic_tree_accepts_puzzle_mismatch_but_puzzle_parser_rejects_it() {
    let bytes =
        include_bytes!("../../../tests/fixtures/rich-analysis/invalid-main-solution-mismatch.json");
    let document: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let puzzle = &document["puzzles"][0];
    parse_analysis_tree(
        &puzzle["analysis"],
        puzzle["fen"].as_str().unwrap(),
        "puzzle fen",
        "test",
    )
    .unwrap();
    assert!(chess_core::parse_puzzle_file(bytes)
        .unwrap_err()
        .contains("legacy solution"));
}
