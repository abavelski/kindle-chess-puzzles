use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, BoardMode, Color,
    Piece, PieceKind, Progress, PromotionChoice, SolutionFeedback,
};
use chess_render::{render, DisplayMetrics};

const PARITY: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn state() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "parity-puzzles.json",
            parse_puzzle_file(PARITY).expect("parity fixture parses"),
        ),
        Progress::new(),
    )
}

fn move_piece(app: &mut AppState, movement: &str) {
    app.dispatch(Action::TapSquare(square(&movement[..2])));
    app.dispatch(Action::TapSquare(square(&movement[2..4])));
}

fn hash(app: &AppState) -> u64 {
    let output = render(app, SCRIBE).expect("render succeeds");
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

fn piece(color: Color, kind: PieceKind) -> Option<Piece> {
    Some(Piece { color, kind })
}

#[test]
fn task05_action_state_render_flows_match_reviewed_snapshots() {
    let mut actual = Vec::new();

    let mut wrong = state();
    let before = wrong.board().clone();
    move_piece(&mut wrong, "d7d8");
    assert_eq!(wrong.board(), &before);
    assert_eq!(wrong.feedback(), SolutionFeedback::Wrong);
    actual.push(("wrong-rollback", hash(&wrong)));

    wrong.dispatch(Action::Reset);
    move_piece(&mut wrong, "d7e8");
    assert_eq!(wrong.feedback(), SolutionFeedback::Complete);
    assert!(wrong.description_visible());
    actual.push(("one-move-complete", hash(&wrong)));

    let mut three = state();
    three.dispatch(Action::NextPuzzle);
    move_piece(&mut three, "e2e6");
    assert_eq!(three.feedback(), SolutionFeedback::Correct);
    assert_eq!(three.solution_ply(), 2);
    assert_eq!(
        three.board().piece_at(square("f8")),
        piece(Color::Black, PieceKind::King)
    );
    actual.push(("three-ply-auto-reply", hash(&three)));

    let mut promotion = state();
    promotion.dispatch(Action::NextPuzzle);
    promotion.dispatch(Action::NextPuzzle);
    move_piece(&mut promotion, "a7a8");
    assert!(promotion.pending_promotion().is_some());
    actual.push(("promotion-modal", hash(&promotion)));
    promotion.dispatch(Action::CancelPromotion);
    assert!(promotion.pending_promotion().is_none());
    assert_eq!(
        promotion.board().piece_at(square("a7")),
        piece(Color::White, PieceKind::Pawn)
    );
    actual.push(("promotion-cancel", hash(&promotion)));

    promotion.dispatch(Action::NextPuzzle);
    move_piece(&mut promotion, "b7b8");
    promotion.dispatch(Action::ChoosePromotion(PromotionChoice::Knight));
    assert_eq!(
        promotion.board().piece_at(square("b8")),
        piece(Color::White, PieceKind::Knight)
    );
    assert_eq!(promotion.feedback(), SolutionFeedback::Complete);
    actual.push(("underpromotion-complete", hash(&promotion)));

    let mut free = state();
    free.dispatch(Action::ToggleMode);
    move_piece(&mut free, "f4a8");
    assert_eq!(free.mode(), BoardMode::FreeBoard);
    assert_eq!(
        free.board().piece_at(square("a8")),
        piece(Color::White, PieceKind::King)
    );
    actual.push(("free-arbitrary-move", hash(&free)));
    free.dispatch(Action::ToggleMode);
    assert_eq!(free.mode(), BoardMode::Solution);
    assert_eq!(
        free.board().piece_at(square("f4")),
        piece(Color::White, PieceKind::King)
    );
    actual.push(("free-return-solution", hash(&free)));

    let mut orientation = state();
    orientation.dispatch(Action::ToggleOrientationLock);
    orientation.dispatch(Action::Flip);
    for _ in 0..4 {
        orientation.dispatch(Action::NextPuzzle);
    }
    assert!(orientation.orientation_locked());
    assert!(orientation.flipped());
    assert_eq!(orientation.active_puzzle().id, "promotion-black-rook");
    actual.push(("lock-flip-navigation", hash(&orientation)));

    let mut description = state();
    description.dispatch(Action::ToggleDescription);
    assert!(description.description_visible());
    assert!(!description.is_current_solved());
    actual.push(("description-before-solve", hash(&description)));

    let mut last = state();
    for _ in 0..10 {
        last.dispatch(Action::NextPuzzle);
    }
    assert_eq!(last.active_puzzle_index(), 5);
    assert!(!last.can_next_puzzle());
    actual.push(("last-next-disabled", hash(&last)));

    const EXPECTED: &[(&str, u64)] = &[
        ("wrong-rollback", 13_599_027_750_844_074_057),
        ("one-move-complete", 12_739_720_276_155_490_974),
        ("three-ply-auto-reply", 9_843_719_431_677_871_167),
        ("promotion-modal", 15_786_100_812_381_706_782),
        ("promotion-cancel", 6_872_069_643_282_936_881),
        ("underpromotion-complete", 2_978_502_556_960_839_776),
        ("free-arbitrary-move", 9_663_979_953_270_859_041),
        ("free-return-solution", 13_318_484_688_087_264_076),
        ("lock-flip-navigation", 13_793_420_379_607_168_895),
        ("description-before-solve", 5_324_232_738_274_091_032),
        ("last-next-disabled", 8_812_294_319_522_785_507),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn parity_fixture_contains_every_checkpoint_c_reference_case() {
    let collection = parse_puzzle_file(PARITY).expect("parity fixture parses");
    let ids = collection
        .puzzles
        .iter()
        .map(|puzzle| puzzle.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        [
            "one-move",
            "three-ply",
            "promotion-white-queen",
            "promotion-white-knight",
            "promotion-black-rook",
            "promotion-auto-reply",
        ]
    );
}
