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
    render(app, SCRIBE)
        .expect("render succeeds")
        .frame
        .checksum64()
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
        ("wrong-rollback", 6_023_108_517_419_401_574),
        ("one-move-complete", 16_169_021_607_660_939_280),
        ("three-ply-auto-reply", 9_263_905_652_670_066_920),
        ("promotion-modal", 17_920_259_070_579_144_423),
        ("promotion-cancel", 6_478_337_335_865_742_286),
        ("underpromotion-complete", 8_659_221_182_538_690_527),
        ("free-arbitrary-move", 8_522_770_154_210_870_921),
        ("free-return-solution", 1_440_368_709_244_297_183),
        ("lock-flip-navigation", 16_449_123_336_200_218_810),
        ("description-before-solve", 4_669_375_466_717_408_664),
        ("last-next-disabled", 6_829_007_319_970_142_874),
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
