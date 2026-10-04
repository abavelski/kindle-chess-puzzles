use chess_core::{parse_puzzle_file, Action, ActiveCollection, AppState, Progress};
use chess_render::{
    render, AnalysisMoveChip, AnalysisMoveChipSource, AnalysisPanelOutput, DisplayMetrics,
};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const RICH: &[u8] = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

const MAIN_ONLY: &[u8] = br#"{
  "version":1,
  "puzzles":[{
    "id":"main-only",
    "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
    "solution":["g6g7"],
    "analysis":{
      "version":1,
      "root":"m0",
      "nodes":[
        {"id":"m0","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","children":["m1"]},
        {"id":"m1","parent":"m0","move":{"uci":"g6g7","san":"Qg7#"},"fen":"7k/6Q1/5K2/8/8/8/8/8 b - - 1 1","role":"main","comment":"The direct finish."}
      ]
    }
  }]
}"#;

const NESTED: &[u8] = br#"{
  "version":1,
  "puzzles":[{
    "id":"nested",
    "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
    "solution":["g6g7"],
    "analysis":{
      "version":1,
      "root":"r0",
      "nodes":[
        {"id":"r0","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","children":["m1","s1"]},
        {"id":"m1","parent":"r0","move":{"uci":"g6g7","san":"Qg7#"},"fen":"7k/6Q1/5K2/8/8/8/8/8 b - - 1 1","role":"main","comment":"Main line."},
        {"id":"s1","parent":"r0","move":{"uci":"g6h6","san":"Qh6"},"fen":"7k/8/5K1Q/8/8/8/8/8 b - - 1 1","role":"sideline","comment":"First side branch.","children":["s2"]},
        {"id":"s2","parent":"s1","move":{"uci":"h8g8","san":"Kg8"},"fen":"6k1/8/5K1Q/8/8/8/8/8 w - - 2 2","role":"sideline","comment":"Nested response."}
      ]
    }
  }]
}"#;

const LONG_COMMENT: &[u8] = br#"{
  "version":1,
  "puzzles":[{
    "id":"long-comment",
    "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
    "solution":["g6g7"],
    "analysis":{
      "version":1,
      "root":"l0",
      "nodes":[
        {"id":"l0","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","children":["l1"]},
        {"id":"l1","parent":"l0","move":{"uci":"g6g7","san":"Qg7#"},"fen":"7k/6Q1/5K2/8/8/8/8/8 b - - 1 1","role":"main","comment":"This deliberately long explanation is wrapped into deterministic rows so the reader can advance through stable pages without scrolling or animation. The same words always occupy the same rows at the same display metrics, which makes page boundaries predictable for e-ink rendering and snapshot review. Additional prose keeps the explanation long enough to require several pages while preserving the board above the status region. Every page remains readable, the close control stays reachable, and previous or next controls appear only when that direction exists. More explanatory prose follows to exercise wrapping across many lines and prove that comments continue safely through the first middle and last pages."}
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

fn long_comment() -> AppState {
    let mut value: serde_json::Value = serde_json::from_slice(LONG_COMMENT).unwrap();
    let comment = value["puzzles"][0]["analysis"]["nodes"][1]["comment"]
        .as_str()
        .unwrap()
        .repeat(4);
    value["puzzles"][0]["analysis"]["nodes"][1]["comment"] = comment.into();
    open(&serde_json::to_vec(&value).unwrap())
}

fn rendered(app: &AppState) -> (u64, AnalysisPanelOutput) {
    let output = render(app, SCRIBE).expect("render succeeds");
    if let Ok(directory) = std::env::var("ANALYSIS_SNAPSHOT_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            format!("{directory}/{}.pgm", output.frame.checksum64()),
            output.frame.to_pgm(),
        )
        .unwrap();
    }
    let checksum = output.frame.checksum64();
    let panel = output.analysis.expect("analysis panel");
    (checksum, panel)
}

fn collect_chips(app: &mut AppState) -> Vec<AnalysisMoveChip> {
    let mut chips = Vec::new();
    loop {
        let output = render(app, SCRIBE).expect("render succeeds");
        let panel = output.analysis.expect("analysis panel");
        chips.extend(panel.move_chips);
        if panel.next_page.is_none() {
            break;
        }
        app.dispatch(Action::AnalysisNextPage);
    }
    chips
}

fn tree_chip_page_hash(app: &mut AppState, node: chess_core::AnalysisNodeIndex) -> u64 {
    loop {
        let output = render(app, SCRIBE).expect("render succeeds");
        let panel = output.analysis.as_ref().expect("analysis panel");
        if panel
            .move_chips
            .iter()
            .any(|chip| chip.source == AnalysisMoveChipSource::TreeMove && chip.node == node)
        {
            if let Ok(directory) = std::env::var("ANALYSIS_SNAPSHOT_DIR") {
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(
                    format!("{directory}/{}.pgm", output.frame.checksum64()),
                    output.frame.to_pgm(),
                )
                .unwrap();
            }
            return output.frame.checksum64();
        }
        assert!(panel.next_page.is_some(), "tree node must appear on a page");
        app.dispatch(Action::AnalysisNextPage);
    }
}

#[test]
fn analysis_starts_with_solution_content_at_the_top_without_a_heading() {
    let output = render(&open(MAIN_ONLY), SCRIBE).unwrap();
    let content = output
        .layout
        .status
        .inset((output.layout.status.height / 12).max(8));
    let panel = output.analysis.unwrap();
    let first = &panel.move_chips[0];
    assert_eq!(
        first.rect.y,
        content.y + 2,
        "only row centering precedes the first move"
    );
    assert_eq!(panel.page_count, 1);
    assert!(panel.previous_page.is_none());
    assert!(panel.next_page.is_none());
}

#[test]
fn reclaimed_heading_space_fits_an_extra_row_without_overflow() {
    let output = render(&open(MAIN_ONLY), SCRIBE).unwrap();
    let content = output
        .layout
        .status
        .inset((output.layout.status.height / 12).max(8));
    let row_height = output.analysis.unwrap().move_chips[0].rect.height + 4;
    let available_rows = (content.height - row_height - 6) / row_height;
    let mut value: serde_json::Value = serde_json::from_slice(MAIN_ONLY).unwrap();
    // One reference per row, followed by the tree's own single move row.
    let spans: Vec<_> = (0..available_rows - 1)
        .flat_map(|_| {
            [
                serde_json::json!({"type":"move_ref","node":"m1"}),
                serde_json::json!({"type":"text","text":"\n"}),
            ]
        })
        .collect();
    value["puzzles"][0]["description_content"] = spans.into();
    value["puzzles"][0]["analysis"]["nodes"][1]["comment"] = "".into();
    let app = open(&serde_json::to_vec(&value).unwrap());
    let output = render(&app, SCRIBE).unwrap();
    let panel = output.analysis.unwrap();
    assert_eq!(
        panel.page_count, 1,
        "the reclaimed row avoids a second page"
    );
    assert_eq!(panel.move_chips.len(), available_rows as usize);
    assert!(panel.next_page.is_none());
    for chip in &panel.move_chips {
        assert!(content.contains_rect(chip.rect));
        assert_eq!(
            panel.hit_test(chip.rect.x, chip.rect.y),
            Some(chess_render::HitTarget::AnalysisMove(chip.node))
        );
    }
}

#[test]
fn rich_analysis_renders_only_explicit_move_chips() {
    let mut app = open(RICH);
    let analysis = app.active_puzzle().analysis.as_ref().expect("analysis");
    let n1 = analysis.node_index("n1").expect("n1");
    let n2 = analysis.node_index("n2").expect("n2");
    let n3 = analysis.node_index("n3").expect("n3");

    let chips = collect_chips(&mut app);
    assert_eq!(chips.len(), 6);
    assert_eq!(
        chips
            .iter()
            .filter(|chip| chip.source == AnalysisMoveChipSource::TreeMove)
            .count(),
        3
    );
    assert_eq!(
        chips
            .iter()
            .filter(|chip| chip.source == AnalysisMoveChipSource::InlineReference)
            .count(),
        3
    );
    assert_eq!(chips.iter().filter(|chip| chip.node == n1).count(), 3);
    assert_eq!(chips.iter().filter(|chip| chip.node == n2).count(), 2);
    assert_eq!(chips.iter().filter(|chip| chip.node == n3).count(), 1);
    assert_eq!(
        chips.iter().filter(|chip| chip.label == "h1=N").count(),
        2,
        "plain prose containing h1=N must not become an extra chip"
    );
    assert!(chips
        .iter()
        .any(|chip| chip.label == "1...h1=Q+" && chip.node == n1));
}

#[test]
fn unselected_tree_moves_and_inline_references_have_no_rectangle() {
    let output = render(&open(RICH), SCRIBE).unwrap();
    let chips = &output.analysis.as_ref().unwrap().move_chips;
    for source in [
        AnalysisMoveChipSource::TreeMove,
        AnalysisMoveChipSource::InlineReference,
    ] {
        let chip = chips.iter().find(|chip| chip.source == source).unwrap();
        assert!(!chip.selected);
        let rect = chip.rect;
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                if x < rect.x + 2
                    || x >= rect.right() - 2
                    || y < rect.y + 2
                    || y >= rect.bottom() - 2
                {
                    assert_eq!(output.frame.pixel(x, y), Some(255), "border of {source:?}");
                }
            }
        }
        assert_eq!(
            output.analysis.as_ref().unwrap().hit_test(rect.x, rect.y),
            Some(chess_render::HitTarget::AnalysisMove(chip.node))
        );
    }
}

#[test]
fn selected_move_is_inverted_and_remains_explicitly_selected() {
    let mut app = open(RICH);
    let n1 = app
        .active_puzzle()
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.node_index("n1"))
        .expect("n1");

    let normal = render(&app, SCRIBE).expect("render succeeds");
    let normal_chip = normal
        .analysis
        .as_ref()
        .expect("analysis")
        .move_chips
        .iter()
        .find(|chip| chip.node == n1)
        .expect("visible n1 chip")
        .clone();

    app.dispatch(Action::SelectAnalysisNode(n1));
    let selected = render(&app, SCRIBE).expect("render succeeds");
    let selected_chip = selected
        .analysis
        .as_ref()
        .expect("analysis")
        .move_chips
        .iter()
        .find(|chip| chip.node == n1)
        .expect("visible selected n1 chip");

    assert_eq!(selected_chip.rect, normal_chip.rect);
    assert!(selected_chip.selected);
    let sample_x = selected_chip.rect.x + 4;
    let sample_y = selected_chip.rect.y + 4;
    assert_eq!(normal.frame.pixel(sample_x, sample_y), Some(255));
    assert_eq!(selected.frame.pixel(sample_x, sample_y), Some(0));
}

#[test]
fn sidelined_continuation_flows_inline_and_all_chips_stay_in_status_region() {
    let mut app = open(NESTED);
    let analysis = app.active_puzzle().analysis.as_ref().expect("analysis");
    let s1 = analysis.node_index("s1").expect("s1");
    let s2 = analysis.node_index("s2").expect("s2");

    let output = render(&app, SCRIBE).expect("render succeeds");
    let status = output.layout.status;
    let mut chips = Vec::new();
    loop {
        let output = render(&app, SCRIBE).expect("render succeeds");
        let panel = output.analysis.expect("analysis");
        for chip in panel.move_chips {
            assert!(status.contains_rect(chip.rect));
            chips.push(chip);
        }
        if panel.next_page.is_none() {
            break;
        }
        app.dispatch(Action::AnalysisNextPage);
    }

    let first = chips
        .iter()
        .find(|chip| chip.source == AnalysisMoveChipSource::TreeMove && chip.node == s1)
        .expect("first sideline chip");
    let nested = chips
        .iter()
        .find(|chip| chip.source == AnalysisMoveChipSource::TreeMove && chip.node == s2)
        .expect("nested sideline chip");
    assert_eq!(nested.rect.y, first.rect.y);
    assert!(nested.rect.x > first.rect.x, "continuation follows inline");
}

#[test]
fn pagination_controls_match_first_middle_and_last_pages() {
    let mut app = long_comment();
    let first = render(&app, SCRIBE).expect("render succeeds");
    let first_panel = first.analysis.expect("analysis");
    assert!(first_panel.page_count >= 3);
    assert_eq!(first_panel.page, 0);
    assert!(first_panel.previous_page.is_none());
    assert!(first_panel.next_page.is_some());
    assert!(first.layout.toolbar.contains_rect(
        first
            .layout
            .control_visual_rect(first.layout.toolbar_targets[0].rect)
    ));

    let middle_page = first_panel.page_count / 2;
    for _ in 0..middle_page {
        app.dispatch(Action::AnalysisNextPage);
    }
    let middle = render(&app, SCRIBE).expect("render succeeds");
    let middle_panel = middle.analysis.expect("analysis");
    assert_eq!(middle_panel.page, middle_page);
    assert!(middle_panel.previous_page.is_some());
    assert!(middle_panel.next_page.is_some());

    for _ in middle_page..first_panel.page_count.saturating_sub(1) {
        app.dispatch(Action::AnalysisNextPage);
    }
    let last = render(&app, SCRIBE).expect("render succeeds");
    let last_panel = last.analysis.expect("analysis");
    assert_eq!(last_panel.page, first_panel.page_count - 1);
    assert!(last_panel.previous_page.is_some());
    assert!(last_panel.next_page.is_none());
}

#[test]
fn no_analysis_toggle_is_inert_and_has_no_panel() {
    let mut app = state(PUZZLES);
    let before = render(&app, SCRIBE).expect("render succeeds");
    app.dispatch(Action::OpenAnalysis);
    let after = render(&app, SCRIBE).expect("render succeeds");

    assert!(!app.analysis_browser_open());
    assert!(after.analysis.is_none());
    assert_eq!(after.frame.checksum64(), before.frame.checksum64());
    assert_eq!(after.frame.checksum64(), 9_482_317_091_365_691_724);
}

#[test]
fn analysis_visual_states_match_reviewed_gray8_snapshots() {
    let mut actual = Vec::new();

    let main = open(MAIN_ONLY);
    actual.push(("main-only", rendered(&main).0));

    let nested = open(NESTED);
    actual.push(("nested-sideline", rendered(&nested).0));

    let rich = open(RICH);
    actual.push(("black-promotion-rich", rendered(&rich).0));

    let mut selected = open(RICH);
    let selected_node = selected
        .active_puzzle()
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.node_index("n1"))
        .expect("n1");
    selected.dispatch(Action::SelectAnalysisNode(selected_node));
    actual.push((
        "selected-move",
        tree_chip_page_hash(&mut selected, selected_node),
    ));

    let first = long_comment();
    let (first_hash, first_panel) = rendered(&first);
    actual.push(("long-comment-first", first_hash));
    assert!(first_panel.page_count >= 3);

    let mut middle = first.clone();
    for _ in 0..first_panel.page_count / 2 {
        middle.dispatch(Action::AnalysisNextPage);
    }
    actual.push(("long-comment-middle", rendered(&middle).0));

    let mut last = first.clone();
    for _ in 0..first_panel.page_count.saturating_sub(1) {
        last.dispatch(Action::AnalysisNextPage);
    }
    actual.push(("long-comment-last", rendered(&last).0));

    let book_bytes = include_bytes!("../../../tests/fixtures/pgn-converter/valid-book.json");
    let mut book = open(book_bytes);
    actual.push(("book-nested-ravs", rendered(&book).0));
    book.dispatch(Action::NextPuzzle);
    book.dispatch(Action::NextPuzzle);
    book.dispatch(Action::OpenAnalysis);
    actual.push(("book-main-only", rendered(&book).0));

    let mut no_analysis = state(PUZZLES);
    no_analysis.dispatch(Action::OpenAnalysis);
    actual.push((
        "no-analysis",
        render(&no_analysis, SCRIBE)
            .expect("render succeeds")
            .frame
            .checksum64(),
    ));

    const EXPECTED: &[(&str, u64)] = &[
        ("main-only", 6_200_642_319_908_229_732),
        ("nested-sideline", 6_869_885_034_225_328_829),
        ("black-promotion-rich", 9_365_800_311_412_063_746),
        ("selected-move", 7_151_119_309_953_166_983),
        ("long-comment-first", 9_064_128_044_607_163_602),
        ("long-comment-middle", 82_013_701_252_302_109),
        ("long-comment-last", 4_298_667_443_637_921_271),
        ("book-nested-ravs", 11_008_779_950_029_694_243),
        ("book-main-only", 7_702_032_852_103_615_242),
        ("no-analysis", 9_482_317_091_365_691_724),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn closed_rich_analysis_entry_matches_reviewed_snapshot() {
    let app = state(RICH);
    let output = render(&app, SCRIBE).unwrap();
    assert_eq!(output.frame.checksum64(), 9_351_849_483_324_201_573);
}
