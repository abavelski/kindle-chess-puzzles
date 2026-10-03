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
        ("white", 12_785_694_883_065_035_435),
        ("black", 3_333_880_099_857_163_200),
        ("selected", 137_963_673_689_345_695),
        ("correct", 4_058_773_257_583_959_064),
        ("wrong", 927_120_475_037_795_186),
        ("complete", 6_635_826_961_927_337_710),
        ("solved", 13_055_497_076_849_099_124),
        ("free-board", 11_201_865_496_429_011_443),
        ("orientation-lock", 3_780_756_930_933_985_335),
        ("description", 5_535_802_974_311_955_834),
        ("promotion", 1_767_314_168_072_193_198),
        ("long-description", 14_666_888_970_708_769_618),
        ("number-difficulty", 7_275_147_851_208_785_662),
        ("collection-picker", 5_311_104_488_793_424_616),
        ("collection-picker-error", 16_232_787_114_710_864_832),
        ("progress-warning", 11_974_942_431_880_536_529),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
