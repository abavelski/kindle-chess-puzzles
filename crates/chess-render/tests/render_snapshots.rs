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
        ("white", 13_976_124_547_179_353_654),
        ("black", 17_389_228_202_895_532_587),
        ("selected", 5_205_851_686_212_523_122),
        ("correct", 1_005_810_583_524_476_053),
        ("wrong", 16_419_888_683_084_111_157),
        ("complete", 49_619_019_944_168_014),
        ("solved", 13_324_217_622_172_422_922),
        ("free-board", 10_287_717_003_122_044_726),
        ("orientation-lock", 9_977_213_342_771_200_358),
        ("description", 4_927_760_156_226_631_382),
        ("promotion", 11_267_433_688_762_715_960),
        ("long-description", 7_122_248_525_340_359_484),
        ("number-difficulty", 14_071_113_336_766_925_544),
        ("collection-picker", 9_453_437_579_279_166_955),
        ("collection-picker-error", 12_176_237_901_934_808_507),
        ("progress-warning", 16_909_000_665_135_762_906),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}
