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
        (y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(normal.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "primary status text should render inside the status panel"
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
        ("white", 7_140_422_721_455_450_270),
        ("black", 10_865_027_921_088_240_638),
        ("selected", 3_591_361_094_147_765_358),
        ("correct", 9_939_301_224_618_320_251),
        ("wrong", 618_933_180_750_850_259),
        ("complete", 14_360_767_832_906_334_727),
        ("solved", 11_012_368_755_925_794_395),
        ("free-board", 16_297_759_552_115_201_675),
        ("orientation-lock", 9_981_317_888_951_315_514),
        ("description", 7_021_364_827_457_628_665),
        ("promotion", 14_831_025_361_946_432_404),
        ("long-description", 855_639_419_083_176_160),
        ("number-difficulty", 335_905_803_665_738_670),
        ("collection-picker", 7_664_530_361_364_253_517),
        ("collection-picker-error", 4_148_872_910_735_820_666),
        ("progress-warning", 9_818_708_924_015_098_271),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
