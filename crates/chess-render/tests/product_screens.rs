use chess_core::{puzzle_app::PuzzleState, review_app::ReviewApp, *};
use chess_render::{render_puzzles, render_review, DisplayMetrics};

#[test]
fn product_screens_render_from_independent_states() {
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    let puzzles = PuzzleState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let game = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap()
    .games
    .remove(0);
    let review = ReviewApp::new(
        vec![ReviewGameEntry::new("games.json", game)],
        WorkspaceSettings::default(),
    );
    assert!(render_puzzles(&puzzles, metrics).is_ok());
    assert!(render_review(&review, metrics).is_ok());
}

#[test]
fn independent_screens_match_legacy_frames_and_map_only_own_actions() {
    use chess_core::{puzzle_app::PuzzleAction as P, review_app::ReviewAction as R};
    use chess_render::{render, HitTarget};
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    for small in [false, true] {
        let collection = || {
            ActiveCollection::from_collection(
                "puzzles.json",
                parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
            )
        };
        let game = parse_review_file(include_bytes!(
            "../../../tests/fixtures/game-review/valid-standard.json"
        ))
        .unwrap()
        .games
        .remove(0);
        let entry = ReviewGameEntry::new("games.json", game);
        let mut preferences = WorkspaceSettings::default();
        preferences.set_small_board(small);
        // Legacy files initialize both profiles from their flat values.
        let legacy_settings =
            Settings::parse(format!("{{\"version\":1,\"small_board\":{small}}}").as_bytes())
                .unwrap();
        let mut legacy =
            AppState::new_with_settings(collection(), Progress::new(), legacy_settings);
        legacy.set_review_games(vec![entry.clone()]);
        let mut puzzle = PuzzleState::new_with_settings(
            collection(),
            Progress::new(),
            preferences.clone(),
            PhysicalBoardSeat::Changing,
        );
        let mut review = ReviewApp::new(vec![entry], preferences);
        for (old, new) in [(Action::ToggleMode, P::ToggleMode), (Action::Flip, P::Flip)] {
            legacy.dispatch(old);
            puzzle.dispatch(new);
        }
        compare_without_legacy_toggle(
            render(&legacy, metrics).unwrap(),
            render_puzzles(&puzzle, metrics).unwrap(),
        );
        legacy.dispatch(Action::ToggleWorkspace);
        legacy.dispatch(Action::ReviewNext);
        review.dispatch(R::ReviewNext);
        let frame = render_review(&review, metrics).unwrap();
        compare_without_legacy_toggle(render(&legacy, metrics).unwrap(), frame.clone());
        let next = frame.layout.next;
        assert_eq!(
            frame.hit_test_review(next.x + next.width / 2, next.y + next.height / 2, &review),
            Some(R::ReviewNext)
        );
    }
    assert_eq!(HitTarget::ReviewNext.into_puzzle_action(), None);
    assert_eq!(HitTarget::Next.into_review_action(), None);
    assert_eq!(HitTarget::SendBoardPosition.into_review_action(), None);
    assert_eq!(HitTarget::ToggleWorkspace.into_puzzle_action(), None);
    assert_eq!(HitTarget::ToggleWorkspace.into_review_action(), None);
    let empty = ReviewApp::new(Vec::new(), WorkspaceSettings::default());
    assert!(render_review(&empty, metrics).is_ok());
}

fn compare_without_legacy_toggle(
    mut legacy: chess_render::RenderOutput,
    mut product: chess_render::RenderOutput,
) {
    // SPLIT-08 deliberately removes the workspace switch from standalone apps.
    let mask = legacy.layout.workspace;
    legacy.frame.fill_rect(mask, 255);
    product.frame.fill_rect(mask, 255);
    assert_eq!(legacy.frame, product.frame);
}

#[test]
fn empty_review_remains_renderable_after_transient_warning_is_cleared() {
    let mut empty = ReviewApp::new(Vec::new(), WorkspaceSettings::default());
    empty.dispatch(chess_core::review_app::ReviewAction::SetTransientMessage(
        None,
    ));
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    assert!(render_review(&empty, metrics).is_ok());
}

#[test]
fn review_persistence_warnings_remain_visible_alongside_authored_analysis() {
    let game = parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap()
    .games
    .remove(0);
    let mut review = ReviewApp::new(
        vec![ReviewGameEntry::new("games.json", game)],
        WorkspaceSettings::default(),
    );
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    let before = render_review(&review, metrics).unwrap();
    review.dispatch(chess_core::review_app::ReviewAction::SetTransientMessage(
        Some("Review resume warning: protected future-schema file preserved.".into()),
    ));
    let after = render_review(&review, metrics).unwrap();
    assert!(after.analysis.is_some());
    assert_ne!(before.frame.checksum64(), after.frame.checksum64());
    let mut replay = before.frame.clone();
    for rect in chess_render::calculate_damage(Some(&before.frame), &after.frame) {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                replay.set_pixel(
                    x.try_into().unwrap(),
                    y.try_into().unwrap(),
                    after.frame.pixel(x, y).unwrap(),
                );
            }
        }
    }
    assert_eq!(replay, after.frame);
    std::fs::write("../../target/review-warning.pgm", after.frame.to_pgm()).unwrap();
}
