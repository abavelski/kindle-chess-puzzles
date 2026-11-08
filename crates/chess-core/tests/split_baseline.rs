//! Frozen persistence boundary for the app split; exercise two separately restored sessions.
use chess_core::*;

fn session(progress: Progress, resume: Option<&ReviewResume>) -> AppState {
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        progress,
    );
    let game = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap()
    .games
    .remove(0);
    app.set_review_games(vec![ReviewGameEntry::new("games.json", game)]);
    if let Some(resume) = resume {
        assert!(app.restore_review_resume(resume).is_none());
    }
    app
}

#[test]
fn split_golden_two_restored_sessions_keep_progress_and_resume_independent() {
    let mut first = session(Progress::new(), None);
    first.dispatch(Action::TapSquare(algebraic_to_square("d7").unwrap()));
    first.dispatch(Action::TapSquare(algebraic_to_square("e8").unwrap()));
    assert!(first.is_current_solved());
    let progress = first.progress().to_bytes().unwrap();
    assert_eq!(
        std::str::from_utf8(&progress).unwrap(),
        include_str!("../../../tests/fixtures/split-baseline/solved-progress.json").trim()
    );
    first.dispatch(Action::ToggleWorkspace);
    first.dispatch(Action::ReviewNext);
    let variation = first
        .active_review_game()
        .unwrap()
        .game()
        .analysis
        .node_index("v2")
        .unwrap();
    first.dispatch(Action::SelectAnalysisNode(variation));
    first.dispatch(Action::Flip);
    let resume = first.review_resume().unwrap().to_bytes().unwrap();
    assert_eq!(first.progress().to_bytes().unwrap(), progress);

    let mut second = session(
        Progress::parse(&progress).unwrap(),
        Some(&ReviewResume::parse(&resume).unwrap()),
    );
    for action in [Action::NextPuzzle, Action::PreviousPuzzle, Action::Reset] {
        second.dispatch(action);
        assert_eq!(second.review_resume().unwrap().to_bytes().unwrap(), resume);
    }
    second.dispatch(Action::ToggleWorkspace);
    for action in [
        Action::ReviewNext,
        Action::ReviewPrevious,
        Action::ReviewToggleFree,
        Action::TapSquare(algebraic_to_square("g1").unwrap()),
        Action::TapSquare(algebraic_to_square("f3").unwrap()),
        Action::ReviewReset,
        Action::ReviewToggleFree,
    ] {
        second.dispatch(action);
        assert_eq!(second.progress().to_bytes().unwrap(), progress);
    }
    second.dispatch(Action::ToggleWorkspace);
    assert!(second.is_current_solved());
    assert_eq!(first.progress().to_bytes().unwrap(), progress);
    assert_eq!(first.review_resume().unwrap().to_bytes().unwrap(), resume);
}
