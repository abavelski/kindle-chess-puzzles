use std::collections::HashSet;

use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AnalysisRole,
    AnalysisTextSpan, AppState, Board, Progress,
};
use chess_render::{render, AnalysisMoveChipSource, DisplayMetrics, HitTarget};

#[test]
fn imported_book_chips_preview_exact_positions_without_changing_solve_or_progress() {
    let collection = parse_puzzle_file(include_bytes!(
        "../../../tests/fixtures/pgn-converter/valid-book.json"
    ))
    .unwrap();
    let mut app = AppState::new(
        ActiveCollection::from_collection("puzzles-book.json", collection),
        Progress::new(),
    );
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    let mut roles = [false; 3];
    for puzzle_index in 0..3 {
        if puzzle_index == 0 {
            // Preview from an in-progress solve, after the automatic opponent reply.
            app.dispatch(Action::TapSquare(algebraic_to_square("e2").unwrap()));
            app.dispatch(Action::TapSquare(algebraic_to_square("e4").unwrap()));
            assert_eq!(app.solution_ply(), 2);
        }
        let live = app.live_board().clone();
        let cursor = app.solution_ply();
        let progress = app.progress().to_bytes().unwrap();
        let puzzle = app.active_puzzle().clone();
        let tree = puzzle.analysis.as_ref().unwrap();
        let count_refs = |spans: &[AnalysisTextSpan]| {
            spans
                .iter()
                .filter(|span| matches!(span, AnalysisTextSpan::MoveRef { .. }))
                .count()
        };
        let expected_refs = count_refs(&puzzle.description_content)
            + tree
                .nodes()
                .iter()
                .map(|node| count_refs(&node.content))
                .sum::<usize>();
        let mut tree_targets = HashSet::new();
        let mut inline_refs = 0;
        assert!(app.dispatch(Action::OpenAnalysis).is_empty());
        loop {
            let output = render(&app, metrics).unwrap();
            let panel = output.analysis.as_ref().unwrap();
            for chip in &panel.move_chips {
                let node = tree.node(chip.node).unwrap();
                match node.role.unwrap() {
                    AnalysisRole::Main => roles[0] = true,
                    AnalysisRole::Alternative => roles[1] = true,
                    AnalysisRole::Sideline => roles[2] = true,
                }
                let action = output
                    .hit_test_app(
                        chip.rect.x + chip.rect.width / 2,
                        chip.rect.y + chip.rect.height / 2,
                        &app,
                    )
                    .and_then(HitTarget::into_action)
                    .unwrap();
                assert_eq!(action, Action::SelectAnalysisNode(chip.node));
                let mut tapped = app.clone();
                assert!(tapped.dispatch(action).is_empty());
                assert_eq!(tapped.board(), &Board::from_fen(&node.fen).unwrap());
                assert_eq!(tapped.live_board(), &live);
                assert_eq!(tapped.solution_ply(), cursor);
                assert_eq!(tapped.progress().to_bytes().unwrap(), progress);
                assert!(tapped.dispatch(Action::CloseAnalysis).is_empty());
                assert_eq!(tapped.board(), &live);
                match chip.source {
                    AnalysisMoveChipSource::TreeMove => {
                        tree_targets.insert(chip.node);
                    }
                    AnalysisMoveChipSource::InlineReference => inline_refs += 1,
                }
            }
            if panel.next_page.is_none() {
                break;
            }
            assert!(app.dispatch(Action::AnalysisNextPage).is_empty());
        }
        assert_eq!(tree_targets.len(), tree.nodes().len() - 1);
        // Plain Nd5/e2e4 and h1=Q+ mentions add no extra interactive targets.
        assert_eq!(inline_refs, expected_refs);
        assert!(app.dispatch(Action::CloseAnalysis).is_empty());
        assert_eq!(app.board(), &live);
        assert_eq!(app.progress().to_bytes().unwrap(), progress);
        if puzzle_index < 2 {
            app.dispatch(Action::NextPuzzle);
        }
    }
    assert!(roles.into_iter().all(|seen| seen));
}
