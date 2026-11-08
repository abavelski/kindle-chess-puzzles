use chess_core::{
    review_app::{ReviewAction as A, ReviewApp},
    *,
};

fn app() -> ReviewApp {
    let game = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap()
    .games
    .remove(0);
    ReviewApp::new(
        vec![ReviewGameEntry::new("games.json", game)],
        WorkspaceSettings::default(),
    )
}

#[test]
fn review_product_constructs_from_games_only_and_restores_variation_resume() {
    let mut original = app();
    original.dispatch(A::ReviewNext);
    let node = original
        .active_review_game()
        .unwrap()
        .game()
        .analysis
        .node_index("v2")
        .unwrap();
    original.dispatch(A::SelectAnalysisNode(node));
    assert_eq!(original.review_state().unwrap().main_line_ply(), 1);
    original.dispatch(A::ReviewFlip);
    let saved = original.review_resume().unwrap();
    let mut restored = app();
    assert!(restored.restore_review_resume(&saved).is_none());
    assert_eq!(restored.board(), original.board());
    assert_eq!(
        restored.review_resume().unwrap().to_bytes().unwrap(),
        saved.to_bytes().unwrap()
    );
    restored.dispatch(A::ReviewNext);
    assert_eq!(restored.review_state().unwrap().main_line_ply(), 2);
}

#[test]
fn review_product_free_is_transient_and_empty_library_surfaces_warning() {
    let mut app = app();
    app.dispatch(A::ReviewNext);
    let authored = app.board().unwrap().clone();
    let saved = app.review_resume().unwrap();
    app.dispatch(A::ReviewToggleFree);
    app.dispatch(A::TapSquare(algebraic_to_square("e4").unwrap()));
    app.dispatch(A::TapSquare(algebraic_to_square("e5").unwrap()));
    assert_ne!(app.board().unwrap(), &authored);
    assert_eq!(app.review_resume(), Some(saved));
    app.dispatch(A::ReviewReset);
    assert_eq!(app.board().unwrap(), &authored);
    app.set_review_games(Vec::new());
    assert!(app.board().is_none());
    assert!(app.transient_message().unwrap().contains("No review games"));
    app.dispatch(A::ReviewNext);
    app.dispatch(A::ReviewToggleFree);
    assert!(app.review_resume().is_none());
}
