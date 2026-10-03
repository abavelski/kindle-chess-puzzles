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
fn header_controls_share_compact_height_and_title_uses_button_scale() {
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
        frame.pixel(layout.refresh.x, refresh_mid_y),
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
    let button_scale = (SCRIBE.dpi / 100).clamp(2, 5);
    let old_header_scale = (SCRIBE.dpi / 75).clamp(2, 6);
    let old_title_y = layout
        .header
        .y
        .saturating_add(layout.header.height.saturating_sub(7 * old_header_scale) / 2);
    let old_only_x = layout
        .refresh
        .right()
        .saturating_add(padding)
        .saturating_add(2 * old_header_scale)
        .saturating_add(2);
    assert_eq!(
        frame.pixel(old_only_x, old_title_y + 1),
        Some(255),
        "title should no longer use the larger header-only font scale"
    );

    let title_y = layout
        .header
        .y
        .saturating_add(layout.header.height.saturating_sub(7 * button_scale) / 2);
    let title_x = layout
        .refresh
        .right()
        .saturating_add(padding)
        .saturating_add(2 * button_scale)
        .saturating_add(1);
    assert_eq!(frame.pixel(title_x, title_y + 1), Some(0));
}

#[test]
fn status_panel_is_rounded_and_shows_one_large_primary_content() {
    let normal = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let status = normal.layout.status;
    let padding = (status.height / 12).max(8);
    let x = status.x + padding;
    let y = status.y + padding;

    assert_eq!(normal.frame.pixel(status.x, status.y), Some(255));
    assert_eq!(normal.frame.pixel(x + 2, y + 2), Some(0));

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
    assert_eq!(described.frame.pixel(x + 9, y + 15), Some(0));

    let mut free_described = state(dot);
    free_described.dispatch(Action::ToggleMode);
    free_described.dispatch(Action::ToggleDescription);
    let free_described = render(&free_described, SCRIBE).expect("render succeeds");
    assert_eq!(free_described.frame.pixel(x, y), Some(255));
    assert_eq!(free_described.frame.pixel(x + 9, y + 15), Some(0));
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

    const EXPECTED: &[(&str, u64)] = &[
        ("white", 12_857_083_097_318_992_116),
        ("black", 10_706_566_293_756_259_236),
        ("selected", 11_498_342_331_450_273_560),
        ("correct", 15_812_716_582_804_702_660),
        ("wrong", 17_707_174_590_654_286_621),
        ("complete", 4_850_085_460_229_043_401),
        ("solved", 9_876_084_892_660_141_491),
        ("free-board", 8_671_163_079_418_578_452),
        ("orientation-lock", 7_746_178_605_488_746_880),
        ("description", 12_189_179_550_342_317_313),
        ("promotion", 10_835_985_786_175_961_452),
        ("long-description", 1_913_185_588_152_541_260),
        ("number-difficulty", 2_363_511_642_772_480_901),
        ("collection-picker", 7_640_832_175_836_636_290),
        ("collection-picker-error", 13_403_774_574_952_154_720),
        ("progress-warning", 13_568_940_307_127_483_054),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
