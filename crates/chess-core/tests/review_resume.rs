use chess_core::*;

fn app() -> AppState {
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let collection = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap();
    let mut other = collection.games[0].clone();
    other.id = "other".into();
    app.set_review_games(vec![
        ReviewGameEntry::new("games-other.json", other),
        ReviewGameEntry::new("games.json", collection.games[0].clone()),
    ]);
    app
}

#[test]
fn restart_restores_game_move_and_orientation_without_changing_puzzles() {
    let mut original = app();
    let progress = original.progress().clone();
    original.dispatch(Action::ToggleWorkspace);
    original.dispatch(Action::OpenReviewGamePicker);
    original.dispatch(Action::SelectReviewGame(1));
    let entry = original.review_games()[1].clone();
    original.dispatch(Action::ActivateReviewGame(
        entry.key().clone(),
        Box::new(entry.game().clone()),
    ));
    original.dispatch(Action::ReviewNext);
    original.dispatch(Action::ReviewNext);
    original.dispatch(Action::Flip);
    original.dispatch(Action::ToggleOrientationLock);
    let saved =
        ReviewResume::parse(&original.review_resume().unwrap().to_bytes().unwrap()).unwrap();
    let mut restored = app();
    assert!(restored.restore_review_resume(&saved).is_none());
    assert_eq!(restored.workspace(), Workspace::Puzzles);
    assert_eq!(restored.progress(), &progress);
    restored.dispatch(Action::ToggleWorkspace);
    assert_eq!(restored.board(), original.board());
    assert_eq!(restored.active_review_game().unwrap().key(), entry.key());
    assert_eq!(restored.review_state(), original.review_state());
    restored.dispatch(Action::ToggleWorkspace);
    assert_eq!(restored.progress(), &progress);
    restored.dispatch(Action::ToggleWorkspace);
    assert_eq!(restored.board(), original.board());
}

#[test]
fn missing_game_and_removed_move_fall_back_safely() {
    let mut original = app();
    original.dispatch(Action::ToggleWorkspace);
    original.dispatch(Action::ReviewNext);
    let mut saved = original.review_resume().unwrap();
    saved.collection_id = "missing.json".into();
    let mut restored = app();
    assert!(restored.restore_review_resume(&saved).is_some());
    assert_eq!(restored.review_state().unwrap().main_line_ply(), 0);
    saved.collection_id = "games-other.json".into();
    saved.move_path.push("a1a8".into());
    assert!(restored.restore_review_resume(&saved).is_some());
    assert_eq!(restored.review_state().unwrap().main_line_ply(), 0);
}

#[test]
fn every_authored_position_restores_after_game_and_node_reordering() {
    let mut original = app();
    original.dispatch(Action::ToggleWorkspace);
    let game = original.active_review_game().unwrap().game().clone();
    for node in game.analysis.nodes() {
        let index = game.analysis.node_index(&node.id).unwrap();
        original.dispatch(Action::SelectAnalysisNode(index));
        let saved = original.review_resume().unwrap();
        // Regeneration may rename node IDs; durable state follows the authored UCI path.
        let source = std::str::from_utf8(include_bytes!(
            "../../../tests/fixtures/game-review/valid-standard.json"
        ))
        .unwrap();
        let mut source = source.to_owned();
        for node in game.analysis.nodes() {
            source = source.replace(
                &format!("\"{}\"", node.id),
                &format!("\"renamed-{}\"", node.id),
            );
        }
        let mut value: serde_json::Value = serde_json::from_str(&source).unwrap();
        value["games"][0]["id"] = "other".into();
        value["games"][0]["analysis"]["nodes"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let regenerated = parse_review_file(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .games
            .remove(0);
        let mut restored = app();
        let other = restored.review_games()[1].clone();
        restored.set_review_games(vec![
            other,
            ReviewGameEntry::new("games-other.json", regenerated),
        ]);
        assert!(
            restored.restore_review_resume(&saved).is_none(),
            "{}",
            node.id
        );
        restored.dispatch(Action::ToggleWorkspace);
        assert_eq!(restored.board(), original.board());
        assert_eq!(
            restored.review_state().unwrap().main_line_ply(),
            original.review_state().unwrap().main_line_ply()
        );
        restored.dispatch(Action::ReviewNext);
        let mut expected = original.clone();
        expected.dispatch(Action::ReviewNext);
        assert_eq!(restored.board(), expected.board());
    }
}

#[test]
fn resume_ignores_free_board_experiments_and_rejects_changed_positions() {
    let mut original = app();
    original.dispatch(Action::ToggleWorkspace);
    original.dispatch(Action::ReviewNext);
    let saved = original.review_resume().unwrap();
    original.dispatch(Action::ToggleMode);
    original.dispatch(Action::TapSquare(algebraic_to_square("e4").unwrap()));
    original.dispatch(Action::TapSquare(algebraic_to_square("e5").unwrap()));
    assert_eq!(original.review_resume(), Some(saved.clone()));
    let mut restored = app();
    restored.restore_review_resume(&saved);
    restored.dispatch(Action::ToggleWorkspace);
    assert!(!restored.review_state().unwrap().free_board_enabled());
    assert_eq!(
        restored.board(),
        original.review_state().unwrap().authored_board()
    );
    let mut changed = saved;
    changed.fen = restored.active_review_game().unwrap().game().fen.clone();
    assert!(restored.restore_review_resume(&changed).is_some());
    assert_eq!(restored.review_state().unwrap().main_line_ply(), 0);
}
