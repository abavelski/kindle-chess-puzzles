use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, CollectionEntry,
    Progress, PromotionChoice,
};
use chess_render::{render, DisplayMetrics};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const PROMOTIONS: &[u8] = include_bytes!("../../../tests/fixtures/promotion-puzzles.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn state(bytes: &[u8]) -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(bytes).expect("fixture parses"),
        ),
        Progress::new(),
    )
}

fn play(app: &mut AppState, movement: &str) {
    app.dispatch(Action::TapSquare(square(&movement[..2])));
    app.dispatch(Action::TapSquare(square(&movement[2..4])));
    if movement.len() == 5 {
        let choice = match movement.as_bytes()[4] {
            b'q' => PromotionChoice::Queen,
            b'r' => PromotionChoice::Rook,
            b'b' => PromotionChoice::Bishop,
            b'n' => PromotionChoice::Knight,
            _ => panic!("bad promotion"),
        };
        app.dispatch(Action::ChoosePromotion(choice));
    }
}

fn hash(app: &AppState) -> u64 {
    let output = render(app, SCRIBE).expect("render succeeds");
    assert_eq!(output.damage.as_slice(), &[output.layout.viewport]);
    if let Ok(directory) = std::env::var("UI_SNAPSHOT_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            format!("{directory}/{}.pgm", output.frame.checksum64()),
            output.frame.to_pgm(),
        )
        .unwrap();
    }
    output.frame.checksum64()
}

#[test]
fn board_outer_frame_is_thin_gray() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let outer = output.layout.board_outer;

    assert_eq!(frame.pixel(outer.x, outer.y), Some(160));
    assert_eq!(frame.pixel(outer.x + 1, outer.y + 1), Some(160));
    assert_eq!(frame.pixel(outer.x + 2, outer.y + 2), Some(255));
}

#[test]
fn refresh_button_uses_vector_refresh_icon() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let refresh = output
        .layout
        .refresh
        .inset(output.layout.header.height / 10);
    let icon = refresh.inset(refresh.width.min(refresh.height) / 4);
    let left = icon.x + icon.width / 5;
    let right = icon.right().saturating_sub(1) - icon.width / 5;
    let top = icon.y + icon.height / 4;
    let bottom = icon.bottom().saturating_sub(1) - icon.height / 4;
    let mid_x = icon.x + icon.width / 2;
    let mid_y = icon.y + icon.height / 2;

    for (x, y) in [(mid_x, top), (right, mid_y), (mid_x, bottom), (left, mid_y)] {
        assert_eq!(frame.pixel(x, y), Some(0), "refresh icon stroke at {x},{y}");
    }
}

#[test]
fn exit_button_uses_vector_close_icon() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let exit = output.layout.exit.inset(output.layout.header.height / 10);
    let icon = exit.inset(exit.width.min(exit.height) / 4);
    let right = icon.right().saturating_sub(1);
    let bottom = icon.bottom().saturating_sub(1);

    for (x, y) in [
        (icon.x, icon.y),
        (right, icon.y),
        (icon.x, bottom),
        (right, bottom),
    ] {
        assert_eq!(frame.pixel(x, y), Some(0), "close icon corner at {x},{y}");
    }
}

#[test]
fn header_controls_share_compact_height_and_title_is_visible() {
    let mut app = state(PUZZLES);
    play(&mut app, "d7e8");
    app.set_collection_entries(vec![CollectionEntry::valid("puzzles.json", "Puzzles")]);

    let output = render(&app, SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let layout = output.layout;
    let inset = layout.header.height / 10;
    let refresh = layout.refresh.inset(inset);
    let files = layout.collection_button.inset(inset);
    let close = layout.exit.inset(inset);
    let solved_width = layout
        .minimum_touch_px()
        .saturating_mul(2)
        .saturating_sub(inset.saturating_mul(2));
    let solved_height = layout.header.height.saturating_sub(inset.saturating_mul(2));

    assert_eq!(files.width, solved_width);
    assert_eq!(files.height, solved_height);
    assert_eq!(refresh.height, solved_height);
    assert_eq!(close.height, solved_height);
    assert_eq!(refresh.width, close.width);
    assert!(close.width < files.width);

    let refresh_mid_y = refresh.y + refresh.height / 2;
    assert_eq!(
        frame.pixel(layout.refresh.x + 4, refresh_mid_y),
        Some(255),
        "refresh visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(refresh.x, refresh_mid_y), Some(0));

    let files_mid_y = files.y + files.height / 2;
    assert_eq!(
        frame.pixel(layout.collection_button.x, files_mid_y),
        Some(255),
        "FILES visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(files.x, files_mid_y), Some(0));

    let close_mid_y = close.y + close.height / 2;
    assert_eq!(
        frame.pixel(layout.exit.x, close_mid_y),
        Some(255),
        "close visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(close.x, close_mid_y), Some(0));

    let padding = layout.header.height / 8;
    let title_start = layout.refresh.right().saturating_add(padding);
    let title_end = layout
        .collection_button
        .x
        .saturating_sub(layout.minimum_touch_px().saturating_mul(2))
        .saturating_sub(padding);
    let title_ink = (layout.header.y + 6..layout.header.bottom().saturating_sub(6))
        .flat_map(|y| (title_start..title_end).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(frame.pixel(x, y), Some(tone) if tone < 160))
        .count();
    assert!(title_ink > 20, "expected rendered title text in the header");
}

#[test]
fn status_panel_is_rounded_and_shows_one_large_primary_content() {
    let normal = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let status = normal.layout.status;
    let padding = (status.height / 12).max(8);
    let x = status.x + padding;
    let y = status.y + padding;

    assert_eq!(normal.frame.pixel(status.x, status.y), Some(255));
    assert!(
        !(y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(normal.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "side to move now appears in the header"
    );

    let dot = br#"{
      "version":1,
      "puzzles":[{
        "id":"dot-description",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "description":".",
        "solution":["g6g7"]
      }]
    }"#;

    let mut described = state(dot);
    described.dispatch(Action::ToggleDescription);
    let described = render(&described, SCRIBE).expect("render succeeds");
    assert_eq!(described.frame.pixel(x, y), Some(255));
    assert!(
        (y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(described.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "description should render visible text inside the status panel"
    );

    let mut free_described = state(dot);
    free_described.dispatch(Action::ToggleMode);
    free_described.dispatch(Action::ToggleDescription);
    let free_described = render(&free_described, SCRIBE).expect("render succeeds");
    assert_eq!(free_described.frame.pixel(x, y), Some(255));
    assert!(
        (y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(free_described.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "description text should remain visible in Free Board mode"
    );
}

fn region_has_ink(output: &chess_render::RenderOutput, rect: chess_render::Rect) -> bool {
    (rect.y..rect.bottom())
        .any(|y| (rect.x..rect.right()).any(|x| output.frame.pixel(x, y).unwrap() < 160))
}

#[test]
fn solution_side_is_centered_in_header_and_absent_from_hidden_description() {
    for black in [false, true] {
        let mut document: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
        if black {
            let fen = document["puzzles"][0]["fen"]
                .as_str()
                .unwrap()
                .replace(" w ", " b ");
            document["puzzles"][0]["fen"] = fen.into();
        }
        let mut app = state(&serde_json::to_vec(&document).unwrap());
        let output = render(&app, SCRIBE).unwrap();
        let center = chess_render::Rect::new(
            SCRIBE.width / 2 - 140,
            10,
            280,
            output.layout.header.height - 20,
        );
        assert!(
            region_has_ink(&output, center),
            "side label should be in header center"
        );
        assert!(
            !region_has_ink(&output, output.layout.status.inset(30)),
            "hidden description should not duplicate side label"
        );
        app.dispatch(Action::Flip);
        let flipped = render(&app, SCRIBE).unwrap();
        for y in center.y..center.bottom() {
            for x in center.x..center.right() {
                assert_eq!(
                    output.frame.pixel(x, y),
                    flipped.frame.pixel(x, y),
                    "Flip must not change solver side"
                );
            }
        }
    }
}

#[test]
fn topic_uses_description_visibility_and_three_lines_fit_with_description() {
    let mut document: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
    document["puzzles"][0]["topic"] =
        serde_json::json!("Геометрический мотив\nКоневые вилки\nУстранение защиты");
    document["puzzles"][0]["description"] = serde_json::json!("A description below the topic.");
    let mut app = state(&serde_json::to_vec(&document).unwrap());
    let hidden = render(&app, SCRIBE).unwrap();
    assert!(!region_has_ink(&hidden, hidden.layout.status.inset(30)));
    app.dispatch(Action::ToggleDescription);
    let revealed = render(&app, SCRIBE).unwrap();
    let rect = revealed.layout.status;
    let padding = (rect.height / 12).max(8);
    let content = rect.inset(padding);
    for line in 0..4 {
        assert!(
            region_has_ink(
                &revealed,
                chess_render::Rect::new(content.x, content.y + line * 47, content.width, 47)
            ),
            "missing topic/description line {line}"
        );
    }
    app.dispatch(Action::ToggleDescription);
    assert_eq!(render(&app, SCRIBE).unwrap().frame, hidden.frame);
    play(&mut app, "d7e8");
    assert!(app.description_visible());
    assert!(region_has_ink(&render(&app, SCRIBE).unwrap(), content));
}

#[test]
fn parity_visual_states_match_reviewed_gray8_snapshots() {
    let mut actual = Vec::new();

    let white = state(PUZZLES);
    actual.push(("white", hash(&white)));

    let mut black = state(PUZZLES);
    black.dispatch(Action::NextPuzzle);
    actual.push(("black", hash(&black)));

    let mut selected = state(PUZZLES);
    selected.dispatch(Action::TapSquare(square("d7")));
    actual.push(("selected", hash(&selected)));

    let mut correct = state(PUZZLES);
    correct.dispatch(Action::NextPuzzle);
    play(&mut correct, "e2e6");
    actual.push(("correct", hash(&correct)));

    let mut wrong = state(PUZZLES);
    play(&mut wrong, "d7d8");
    actual.push(("wrong", hash(&wrong)));

    let mut complete = state(PUZZLES);
    play(&mut complete, "d7e8");
    actual.push(("complete", hash(&complete)));

    let mut solved = complete.clone();
    solved.dispatch(Action::Reset);
    actual.push(("solved", hash(&solved)));

    let mut free = state(PUZZLES);
    free.dispatch(Action::ToggleMode);
    actual.push(("free-board", hash(&free)));

    let mut locked = state(PUZZLES);
    locked.dispatch(Action::ToggleOrientationLock);
    actual.push(("orientation-lock", hash(&locked)));

    let mut described = state(PUZZLES);
    described.dispatch(Action::ToggleDescription);
    actual.push(("description", hash(&described)));

    let mut promotion = state(PROMOTIONS);
    promotion.dispatch(Action::TapSquare(square("a7")));
    promotion.dispatch(Action::TapSquare(square("a8")));
    actual.push(("promotion", hash(&promotion)));

    let long = br#"{
      "version":1,
      "puzzles":[{
        "id":"long-description",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "description":"A long first line explaining the tactical idea.\nA second line wraps across the deterministic description region so clipping is visible.",
        "difficulty":"Hard",
        "solution":["g6g7"]
      }]
    }"#;
    let mut long_description = state(long);
    long_description.dispatch(Action::ToggleDescription);
    actual.push(("long-description", hash(&long_description)));

    let number = br#"{
      "version":1,
      "puzzles":[{
        "id":"number-difficulty",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "difficulty":2.5,
        "solution":["g6g7"]
      }]
    }"#;
    actual.push(("number-difficulty", hash(&state(number))));

    let mut picker = state(PUZZLES);
    picker.set_collection_entries(vec![
        CollectionEntry::valid("puzzles.json", "Lichess sample puzzles"),
        CollectionEntry::valid("puzzles-endgames.json", "Endgames"),
    ]);
    picker.dispatch(Action::OpenCollectionPicker);
    actual.push(("collection-picker", hash(&picker)));

    let mut picker_error = state(PUZZLES);
    picker_error.set_collection_entries(vec![
        CollectionEntry::valid("puzzles.json", "Lichess sample puzzles"),
        CollectionEntry::invalid("puzzles-broken.json", "Invalid progress fixture"),
    ]);
    picker_error.dispatch(Action::OpenCollectionPicker);
    actual.push(("collection-picker-error", hash(&picker_error)));

    let mut warning = state(PUZZLES);
    warning.dispatch(Action::SetTransientMessage(Some(
        "Progress warning: write failed; newest progress remains in memory.".to_owned(),
    )));
    actual.push(("progress-warning", hash(&warning)));

    let mut topic_json: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
    topic_json["puzzles"][0]["topic"] =
        serde_json::json!("Геометрический мотив\nКоневые вилки\nУстранение защиты");
    let mut topic = state(&serde_json::to_vec(&topic_json).unwrap());
    actual.push(("topic-hidden", hash(&topic)));
    let mut black_topic_json = topic_json.clone();
    black_topic_json["puzzles"][0]["fen"] =
        serde_json::json!("8/3B2pp/p5k1/6P1/1ppp1K2/8/1P6/8 b - - 0 39");
    actual.push((
        "black-side-header",
        hash(&state(&serde_json::to_vec(&black_topic_json).unwrap())),
    ));
    topic.dispatch(Action::ToggleDescription);
    actual.push(("topic-three-lines", hash(&topic)));
    topic.dispatch(Action::ToggleMode);
    topic.dispatch(Action::ToggleDescription);
    actual.push(("topic-free-board", hash(&topic)));
    topic_json["puzzles"][0]["description"] = serde_json::Value::Null;
    let mut topic_only = state(&serde_json::to_vec(&topic_json).unwrap());
    topic_only.dispatch(Action::ToggleDescription);
    actual.push(("topic-only", hash(&topic_only)));
    topic_json["puzzles"][0]["id"] = serde_json::json!(
        "an-extremely-long-puzzle-identifier-that-must-not-overlap-the-centered-side-label"
    );
    actual.push((
        "long-header",
        hash(&state(&serde_json::to_vec(&topic_json).unwrap())),
    ));

    const EXPECTED: &[(&str, u64)] = &[
        ("white", 9_482_317_091_365_691_724),
        ("black", 15_129_938_540_338_867_176),
        ("selected", 10_554_746_923_683_061_948),
        ("correct", 2_663_393_823_120_754_628),
        ("wrong", 9_059_133_726_631_875_749),
        ("complete", 8_236_664_936_829_541_307),
        ("solved", 1_574_159_436_012_161_577),
        ("free-board", 10_394_563_463_216_060_619),
        ("orientation-lock", 13_753_266_639_184_449_212),
        ("description", 7_859_123_114_726_742_969),
        ("promotion", 14_337_950_829_846_745_346),
        ("long-description", 18_226_650_496_557_486_600),
        ("number-difficulty", 1_303_463_855_139_560_660),
        ("collection-picker", 4_877_646_207_269_875_407),
        ("collection-picker-error", 14_934_673_025_716_135_696),
        ("progress-warning", 5_622_647_288_038_611_571),
        ("topic-hidden", 9_482_317_091_365_691_724),
        ("black-side-header", 13_560_601_721_915_842_894),
        ("topic-three-lines", 12_009_366_906_417_777_708),
        ("topic-free-board", 11_512_634_764_333_795_092),
        ("topic-only", 967_484_523_730_318_318),
        ("long-header", 17_090_751_274_210_726_857),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
