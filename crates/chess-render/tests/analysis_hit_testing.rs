use chess_core::{parse_puzzle_file, Action, ActiveCollection, AppState, Board, Progress};
use chess_render::{
    calculate_damage, compact_damage, render, AnalysisMoveChipSource, DisplayMetrics, HitTarget,
    Rect,
};

const RICH: &[u8] = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

const ADJACENT_REFS: &[u8] = br#"{
  "version":1,
  "puzzles":[{
    "id":"adjacent-refs",
    "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
    "description_content":[
      {"type":"move_ref","node":"m1","label":"Qg7#"},
      {"type":"move_ref","node":"s1","label":"Qh6"}
    ],
    "solution":["g6g7"],
    "analysis":{
      "version":1,
      "root":"r0",
      "nodes":[
        {"id":"r0","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","children":["m1","s1"]},
        {"id":"m1","parent":"r0","move":{"uci":"g6g7","san":"Qg7#"},"fen":"7k/6Q1/5K2/8/8/8/8/8 b - - 1 1","role":"main"},
        {"id":"s1","parent":"r0","move":{"uci":"g6h6","san":"Qh6"},"fen":"7k/8/5K1Q/8/8/8/8/8 b - - 1 1","role":"sideline"}
      ]
    }
  }]
}"#;

fn state(bytes: &[u8]) -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(bytes).expect("fixture parses"),
        ),
        Progress::new(),
    )
}

fn open(bytes: &[u8]) -> AppState {
    let mut app = state(bytes);
    app.dispatch(Action::OpenAnalysis);
    assert!(app.analysis_browser_open());
    app
}

fn center(rect: Rect) -> (u32, u32) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}

#[test]
fn every_visible_tree_and_inline_chip_selects_its_exact_precomputed_position() {
    let mut app = open(RICH);
    let mut tree_hits = 0;
    let mut inline_hits = 0;
    let mut h1_knight_chips = 0;

    loop {
        let output = render(&app, SCRIBE).expect("render succeeds");
        let panel = output.analysis.as_ref().expect("analysis panel");

        for chip in &panel.move_chips {
            assert!(chip.hit_rect.contains_rect(chip.rect));
            assert!(
                chip.hit_rect.width > chip.rect.width || chip.hit_rect.height > chip.rect.height,
                "move chips must have padding outside their visual rectangle"
            );

            let (x, y) = center(chip.rect);
            let target = output.hit_test_app(x, y, &app);
            assert_eq!(target, Some(HitTarget::AnalysisMove(chip.node)));
            assert_eq!(
                target.and_then(HitTarget::into_action),
                Some(Action::SelectAnalysisNode(chip.node))
            );

            let expected = {
                let analysis = app.active_puzzle().analysis.as_ref().expect("analysis");
                let node = analysis.node(chip.node).expect("chip target exists");
                Board::from_fen(&node.fen).expect("stored analysis FEN parses")
            };
            let mut tapped = app.clone();
            tapped.dispatch(Action::SelectAnalysisNode(chip.node));
            assert_eq!(tapped.board(), &expected);

            match chip.source {
                AnalysisMoveChipSource::TreeMove => tree_hits += 1,
                AnalysisMoveChipSource::InlineReference => inline_hits += 1,
            }
            if chip.label == "h1=N" {
                h1_knight_chips += 1;
            }
        }

        if panel.next_page.is_none() {
            break;
        }
        app.dispatch(Action::AnalysisNextPage);
    }

    assert_eq!(tree_hits, 3);
    assert_eq!(inline_hits, 3);
    assert_eq!(
        h1_knight_chips, 2,
        "the plain-text h1=N mention must not create a third target"
    );
}

#[test]
fn adjacent_padded_targets_resolve_to_the_chip_that_is_visibly_under_the_tap() {
    let app = open(ADJACENT_REFS);
    let output = render(&app, SCRIBE).expect("render succeeds");
    let panel = output.analysis.as_ref().expect("analysis panel");
    let inline = panel
        .move_chips
        .iter()
        .filter(|chip| chip.source == AnalysisMoveChipSource::InlineReference)
        .collect::<Vec<_>>();
    assert_eq!(inline.len(), 2);
    assert!(inline[0].hit_rect.intersects(inline[1].hit_rect));
    assert_eq!(inline[0].rect.right(), inline[1].rect.x);

    let y = inline[0].rect.y + inline[0].rect.height / 2;
    assert_eq!(
        output.hit_test_app(inline[0].rect.right() - 1, y, &app),
        Some(HitTarget::AnalysisMove(inline[0].node))
    );
    assert_eq!(
        output.hit_test_app(inline[1].rect.x, y, &app),
        Some(HitTarget::AnalysisMove(inline[1].node))
    );
}

#[test]
fn analysis_controls_dispatch_and_board_taps_are_disabled_without_moving_old_targets() {
    let mut app = open(RICH);
    let first = render(&app, SCRIBE).expect("render succeeds");
    let first_panel = first.analysis.as_ref().expect("analysis panel");

    let board_point = center(first.layout.square_rect(0));
    assert_eq!(first.hit_test_app(board_point.0, board_point.1, &app), None);

    for control in first.layout.toolbar_targets {
        let point = center(control.rect);
        assert_eq!(
            first.hit_test_app(point.0, point.1, &app),
            Some(control.target)
        );
    }
    for (rect, target) in [
        (first.layout.previous, HitTarget::Previous),
        (first.layout.next, HitTarget::Next),
        (first.layout.refresh, HitTarget::Refresh),
        (first.layout.exit, HitTarget::Exit),
    ] {
        let point = center(rect);
        assert_eq!(first.hit_test_app(point.0, point.1, &app), Some(target));
    }

    let next = first_panel.next_page.expect("rich fixture spans pages");
    let next_point = center(next);
    let next_target = first
        .hit_test_app(next_point.0, next_point.1, &app)
        .expect("next-page target");
    assert_eq!(next_target, HitTarget::AnalysisNextPage);
    app.dispatch(
        next_target
            .into_action()
            .expect("page target maps to action"),
    );
    assert_eq!(app.analysis_page(), 1);

    let second = render(&app, SCRIBE).expect("render succeeds");
    let second_panel = second.analysis.as_ref().expect("analysis panel");
    let previous = second_panel.previous_page.expect("previous-page target");
    let previous_point = center(previous);
    assert_eq!(
        second.hit_test_app(previous_point.0, previous_point.1, &app),
        Some(HitTarget::AnalysisPreviousPage)
    );

    let close_point = center(second_panel.close);
    let close_target = second
        .hit_test_app(close_point.0, close_point.1, &app)
        .expect("close target");
    assert_eq!(close_target, HitTarget::CloseAnalysis);
    app.dispatch(close_target.into_action().expect("close maps to action"));
    assert!(!app.analysis_browser_open());
    assert_eq!(app.board(), app.live_board());
}

#[test]
fn preview_selection_damage_is_regional_to_board_and_analysis_panel() {
    let mut app = open(RICH);
    let before = render(&app, SCRIBE).expect("render succeeds");
    let chip = before
        .analysis
        .as_ref()
        .expect("analysis panel")
        .move_chips
        .first()
        .expect("visible move chip");
    let point = center(chip.rect);
    let action = before
        .hit_test_app(point.0, point.1, &app)
        .and_then(HitTarget::into_action)
        .expect("move chip maps to action");
    app.dispatch(action);

    let after = render(&app, SCRIBE).expect("render succeeds");
    let damage = compact_damage(
        &calculate_damage(Some(&before.frame), &after.frame),
        after.layout,
    );

    assert!(!damage.is_empty());
    assert!(!damage.contains(&after.layout.viewport));
    assert!(
        damage
            .iter()
            .any(|rect| rect.intersects(after.layout.board_outer)),
        "preview must update changed board squares"
    );
    assert!(
        damage
            .iter()
            .any(|rect| rect.intersects(after.layout.status)),
        "selection treatment must update the analysis panel"
    );
    assert!(
        damage.iter().all(|rect| {
            rect.intersects(after.layout.board_outer) || rect.intersects(after.layout.status)
        }),
        "preview selection should not dirty unrelated header/toolbar/navigation regions: {damage:?}"
    );
}

#[test]
fn rich_analysis_has_a_visible_entry_control_with_a_working_action() {
    let mut app = state(RICH);
    let output = render(&app, SCRIBE).unwrap();
    let rect = output
        .analysis_entry
        .expect("rich puzzle exposes analysis entry");
    let (x, y) = center(rect);
    let target = output.hit_test_app(x, y, &app).unwrap();
    assert_eq!(target.into_action(), Some(Action::OpenAnalysis));
    app.dispatch(target.into_action().unwrap());
    assert!(render(&app, SCRIBE).unwrap().analysis.is_some());
    let legacy = state(include_bytes!("../../../tests/fixtures/puzzles.json"));
    assert!(render(&legacy, SCRIBE).unwrap().analysis_entry.is_none());
}
