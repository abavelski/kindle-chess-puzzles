//! Task 48: run the host converter, then exercise its output through core and rendering.
use chess_core::{
    algebraic_to_square, parse_puzzle_file, parse_review_file, Action, ActiveCollection,
    AnalysisNodeIndex, AppState, Board, Effect, Progress, ReviewGameEntry, Workspace,
};
use chess_render::{
    calculate_damage, compact_damage, render, AnalysisMoveChipSource, DisplayMetrics, Gray8,
    HitTarget, Rect,
};
use std::{collections::HashSet, path::PathBuf, process::Command};

const METRICS: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn imported_state() -> AppState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("python3")
        .current_dir(&root)
        .args([
            "tools/pgn_review_converter.py",
            "tests/fixtures/pgn-review-converter/annotated-unicode.pgn",
        ])
        .output()
        .expect("host Python with requirements-tools.txt installed");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        std::fs::read(root.join("tests/fixtures/pgn-review-converter/annotated-unicode.json"))
            .expect("reviewed converter fixture")
    );
    let collection =
        parse_review_file(&output.stdout).expect("converter output passes runtime validation");
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    // Leave a graded three-ply puzzle in progress after its automatic reply.
    app.dispatch(Action::NextPuzzle);
    app.dispatch(tap("e2"));
    app.dispatch(tap("e6"));
    assert_eq!(app.solution_ply(), 2);
    app.dispatch(Action::ToggleOrientationLock);
    app.dispatch(Action::Flip);
    app.dispatch(Action::ToggleDescription);
    app.set_review_games(
        collection
            .games
            .into_iter()
            .map(|game| ReviewGameEntry::new("games-classics.json", game))
            .collect(),
    );
    app
}

fn tap(square: &str) -> Action {
    Action::TapSquare(algebraic_to_square(square).unwrap())
}

fn transition(app: &mut AppState, action: Action) -> Gray8 {
    let previous = render(app, METRICS).unwrap().frame;
    let effects = app.dispatch(action);
    assert!(
        !effects.contains(&Effect::ProgressChanged),
        "review must not save puzzle progress"
    );
    let current = render(app, METRICS).unwrap();
    let damage = compact_damage(
        &calculate_damage(Some(&previous), &current.frame),
        current.layout,
    );
    assert!(
        !damage.contains(&current.layout.viewport),
        "review transitions should retain regional damage"
    );
    let mut replay = previous;
    for rect in damage {
        assert!(current.layout.viewport.contains_rect(rect));
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                replay.set_pixel(x as i32, y as i32, current.frame.pixel(x, y).unwrap());
            }
        }
    }
    assert_eq!(
        replay, current.frame,
        "regional presentation reconstructs all pixels"
    );
    current.frame
}

fn click(app: &mut AppState, rect: Rect, expected: HitTarget) {
    let output = render(app, METRICS).unwrap();
    let target = output.hit_test_app(rect.x + rect.width / 2, rect.y + rect.height / 2, app);
    assert_eq!(target, Some(expected));
    transition(app, target.unwrap().into_action().unwrap());
}

fn assert_position(app: &AppState, index: AnalysisNodeIndex, ply: usize) {
    let game = app.active_review_game().unwrap().game();
    assert_eq!(app.selected_analysis_node(), Some(index));
    assert_eq!(app.review_state().unwrap().main_line_ply(), ply);
    assert_eq!(
        app.board(),
        &Board::from_fen(&game.analysis.node(index).unwrap().fen).unwrap()
    );
}

#[test]
fn annotated_import_traversal_targets_scratch_and_puzzle_restore() {
    for small in [false, true] {
        let mut app = imported_state();
        if small {
            app.dispatch(Action::ToggleBoardSizeSetting);
        }
        let puzzle = app.clone();
        let puzzle_frame = render(&app, METRICS).unwrap().frame;
        let progress = app.progress().to_bytes().unwrap();
        transition(&mut app, Action::ToggleWorkspace);
        assert_eq!(app.workspace(), Workspace::Review);
        let game = app.active_review_game().unwrap().game().clone();
        let main = game.analysis.main_line_nodes();
        assert_eq!(main.len(), 7);
        assert_position(&app, main[0], 0);
        assert!(!app.review_state().unwrap().can_previous());
        transition(&mut app, Action::ReviewPrevious);
        for (ply, index) in main.iter().enumerate().skip(1) {
            let rect = render(&app, METRICS).unwrap().layout.next;
            click(&mut app, rect, HitTarget::ReviewNext);
            assert_position(&app, *index, ply);
        }
        assert!(!app.review_state().unwrap().can_next(&game));
        transition(&mut app, Action::ReviewNext);
        for ply in (0..main.len() - 1).rev() {
            let rect = render(&app, METRICS).unwrap().layout.previous;
            click(&mut app, rect, HitTarget::ReviewPrevious);
            assert_position(&app, main[ply], ply);
        }

        // Walk all rendered pages: every tree move and the explicit prose reference is tappable.
        let mut tree_nodes = HashSet::new();
        let mut refs = 0;
        loop {
            let output = render(&app, METRICS).unwrap();
            let panel = output.analysis.unwrap();
            for chip in panel.move_chips {
                let mut preview = app.clone();
                click(&mut preview, chip.rect, HitTarget::AnalysisMove(chip.node));
                let anchor = game.analysis.nearest_main_line_ancestor(chip.node).unwrap();
                let ply = game.analysis.main_line_ply(anchor).unwrap();
                assert_position(&preview, chip.node, ply);
                let selected = render(&preview, METRICS).unwrap();
                assert!(selected
                    .analysis
                    .unwrap()
                    .move_chips
                    .iter()
                    .any(|c| c.node == chip.node && c.selected));
                if chip.source == AnalysisMoveChipSource::TreeMove {
                    tree_nodes.insert(chip.node);
                } else {
                    refs += 1;
                }
                if !game.analysis.is_main_line(chip.node) {
                    let mut backwards = preview.clone();
                    transition(&mut backwards, Action::ReviewPrevious);
                    assert_position(&backwards, main[ply - 1], ply - 1);
                    transition(&mut preview, Action::ReviewNext);
                    assert_position(&preview, main[ply + 1], ply + 1);
                }
            }
            let Some(rect) = panel.next_page else {
                break;
            };
            click(&mut app, rect, HitTarget::AnalysisNextPage);
            assert!(app.analysis_page() < 32);
        }
        assert_eq!(tree_nodes.len(), game.analysis.nodes().len() - 1);
        assert_eq!(refs, 1, "plain Nc3 remains inert");

        transition(&mut app, Action::SelectAnalysisNode(main[1]));
        let authored = app.board().clone();
        transition(&mut app, Action::ReviewToggleFree);
        assert_eq!(app.board(), &authored);
        transition(&mut app, tap("g1"));
        transition(&mut app, tap("f3"));
        assert_ne!(app.board(), &authored);
        transition(&mut app, Action::ReviewReset);
        assert_eq!(app.board(), &authored);
        transition(&mut app, tap("g1"));
        transition(&mut app, tap("f3"));
        let variation = game
            .analysis
            .nodes()
            .iter()
            .find_map(|node| {
                let index = game.analysis.node_index(&node.id).unwrap();
                (!game.analysis.is_main_line(index)).then_some(index)
            })
            .unwrap();
        transition(&mut app, Action::SelectAnalysisNode(variation));
        assert_eq!(app.board(), app.review_state().unwrap().authored_board());
        transition(&mut app, Action::ReviewNext);
        assert_eq!(app.board(), app.review_state().unwrap().authored_board());
        assert!(app.review_state().unwrap().free_board_enabled());
        transition(&mut app, tap("g1"));
        transition(&mut app, tap("f3"));
        assert_ne!(app.board(), app.review_state().unwrap().authored_board());
        transition(&mut app, Action::ReviewToggleFree);
        assert_eq!(app.board(), app.review_state().unwrap().authored_board());
        assert!(!app.review_state().unwrap().free_board_enabled());
        assert_eq!(app.active_review_game().unwrap().game(), &game);
        transition(&mut app, Action::ToggleWorkspace);
        assert_eq!(app.progress().to_bytes().unwrap(), progress);
        assert_eq!(app.live_board(), puzzle.live_board());
        assert_eq!(app.solution_ply(), puzzle.solution_ply());
        assert_eq!(app.feedback(), puzzle.feedback());
        assert_eq!(app.active_puzzle_index(), puzzle.active_puzzle_index());
        assert_eq!(app.flipped(), puzzle.flipped());
        assert_eq!(app.orientation_locked(), puzzle.orientation_locked());
        assert_eq!(app.description_visible(), puzzle.description_visible());
        assert_eq!(render(&app, METRICS).unwrap().frame, puzzle_frame);
    }
}

#[test]
fn imported_standard_and_small_snapshots() {
    let mut app = imported_state();
    app.dispatch(Action::ToggleWorkspace);
    let mut hashes = Vec::new();
    for small in [false, true] {
        if small {
            app.dispatch(Action::ToggleBoardSizeSetting);
        }
        let output = render(&app, METRICS).unwrap();
        hashes.push(output.frame.checksum64());
        // Inspectable Gray8 artifact for intentional snapshot review.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target");
        std::fs::write(
            root.join(if small {
                "review-import-small.pgm"
            } else {
                "review-import-standard.pgm"
            }),
            output.frame.to_pgm(),
        )
        .unwrap();
    }
    // Reviewed at 1860x2480: Unicode identity, nested RAVs, NAG, explicit bold
    // reference beside plain Nc3, disabled PREV, and larger SMALL movetext.
    assert_eq!(
        hashes,
        vec![17_427_706_974_452_155_198, 10_621_961_546_106_974_293]
    );
}
