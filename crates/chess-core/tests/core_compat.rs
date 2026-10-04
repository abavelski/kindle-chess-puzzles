use chess_core::{
    algebraic_to_square, parse_fen, parse_puzzle_file, parse_uci_move,
    sorted_puzzle_collection_filenames, square_to_algebraic, Board, Color, Piece, PieceKind,
    Progress, TapResult, UciMove, MAX_PUZZLE_FILE_BYTES,
};
use serde_json::json;

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const PROMOTIONS: &[u8] = include_bytes!("../../../tests/fixtures/promotion-puzzles.json");
const ENDGAMES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles-endgames.json");
const PROGRESS: &[u8] = include_bytes!("../../../tests/fixtures/progress.v1.json");

fn piece(color: Color, kind: PieceKind) -> Option<Piece> {
    Some(Piece { color, kind })
}

fn sample_collection() -> serde_json::Value {
    json!({
        "version": 1,
        "puzzles": [{
            "id": "sample",
            "fen": "7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
            "solution": ["g6g7"]
        }]
    })
}

fn parse_json(value: serde_json::Value) -> Result<chess_core::PuzzleCollection, String> {
    parse_puzzle_file(&serde_json::to_vec(&value).expect("JSON test fixture encodes"))
}

fn solution(puzzle: &chess_core::Puzzle) -> Vec<&str> {
    puzzle.solution.iter().map(String::as_str).collect()
}

#[test]
fn starting_board_uses_a8_zero_and_h1_sixty_three() {
    let board = Board::starting_position();
    assert_eq!(board.piece_at(0), piece(Color::Black, PieceKind::Rook));
    assert_eq!(board.piece_at(7), piece(Color::Black, PieceKind::Rook));
    assert_eq!(board.piece_at(8), piece(Color::Black, PieceKind::Pawn));
    assert_eq!(board.piece_at(48), piece(Color::White, PieceKind::Pawn));
    assert_eq!(board.piece_at(56), piece(Color::White, PieceKind::Rook));
    assert_eq!(board.piece_at(63), piece(Color::White, PieceKind::Rook));
    assert_eq!(board.piece_at(27), None);
}

#[test]
fn fen_parses_piece_placement_and_active_color() {
    let fen = parse_fen("7k/8/5KQ1/8/8/8/8/8 w - - 0 1").expect("valid FEN");
    assert_eq!(fen.active_color(), Color::White);
    assert_eq!(fen.piece_at(7), piece(Color::Black, PieceKind::King));
    assert_eq!(fen.piece_at(21), piece(Color::White, PieceKind::King));
    assert_eq!(fen.piece_at(22), piece(Color::White, PieceKind::Queen));

    let black = parse_fen("8/8/8/8/8/5kq1/8/7K b - - 0 1").expect("valid FEN");
    assert_eq!(black.active_color(), Color::Black);
}

#[test]
fn fen_rejects_invalid_fields_ranks_pieces_and_metadata() {
    for invalid in [
        "8/8/8/8/8/8/8/8 w - - 0",
        "8/8/8/8/8/8/8 w - - 0 1",
        "9/8/8/8/8/8/8/8 w - - 0 1",
        "7x/8/8/8/8/8/8/8 w - - 0 1",
        "8/8/8/8/8/8/8/8 x - - 0 1",
        "8/8/8/8/8/8/8/8 w KK - 0 1",
        "8/8/8/8/8/8/8/8 w - e3 0 1",
        "8/8/8/8/8/8/8/8 b - e6 0 1",
        "8/8/8/8/8/8/8/8 w - - nope 1",
        "8/8/8/8/8/8/8/8 w - - 0 0",
    ] {
        assert!(
            parse_fen(invalid).is_err(),
            "unexpectedly accepted: {invalid}"
        );
    }
}

#[test]
fn taps_distinguish_selection_deselection_and_completed_move() {
    let mut board = Board::default();
    assert_eq!(board.tap(32), TapResult::NoChange);
    assert_eq!(board.tap(52), TapResult::SelectionChanged);
    assert_eq!(board.selected(), Some(52));
    assert_eq!(board.tap(52), TapResult::SelectionChanged);
    assert_eq!(board.selected(), None);

    assert_eq!(board.tap(52), TapResult::SelectionChanged);
    assert_eq!(board.tap(36), TapResult::Moved { from: 52, to: 36 });
    assert_eq!(board.piece_at(52), None);
    assert_eq!(board.piece_at(36), piece(Color::White, PieceKind::Pawn));
}

#[test]
fn moving_onto_an_occupied_square_replaces_the_piece() {
    let mut board = Board::default();
    assert_eq!(board.tap(56), TapResult::SelectionChanged);
    assert_eq!(board.tap(0), TapResult::Moved { from: 56, to: 0 });
    assert_eq!(board.piece_at(0), piece(Color::White, PieceKind::Rook));
    assert_eq!(board.piece_at(56), None);
}

#[test]
fn white_and_black_promotions_are_staged_without_moving() {
    let mut white = Board::from_fen("7k/P7/8/8/8/8/8/K7 w - - 0 1").expect("valid FEN");
    assert_eq!(white.tap(8), TapResult::SelectionChanged);
    assert_eq!(
        white.tap(0),
        TapResult::Promotion {
            from: 8,
            to: 0,
            color: Color::White
        }
    );
    assert_eq!(white.piece_at(8), piece(Color::White, PieceKind::Pawn));
    assert_eq!(white.piece_at(0), None);

    let mut black = Board::from_fen("k7/8/8/8/8/8/p7/7K b - - 0 1").expect("valid FEN");
    assert_eq!(black.tap(48), TapResult::SelectionChanged);
    assert_eq!(
        black.tap(56),
        TapResult::Promotion {
            from: 48,
            to: 56,
            color: Color::Black
        }
    );
    assert_eq!(black.piece_at(48), piece(Color::Black, PieceKind::Pawn));
    assert_eq!(black.piece_at(56), None);
}

#[test]
fn atomic_promotion_supports_q_r_b_n_only() {
    for kind in [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
    ] {
        let mut board = Board::from_fen("7k/P7/8/8/8/8/8/K7 w - - 0 1").expect("valid FEN");
        assert!(board.promote_pawn(8, 0, kind));
        assert_eq!(board.piece_at(8), None);
        assert_eq!(board.piece_at(0), piece(Color::White, kind));
    }

    for kind in [PieceKind::Pawn, PieceKind::King] {
        let mut board = Board::from_fen("7k/P7/8/8/8/8/8/K7 w - - 0 1").expect("valid FEN");
        assert!(!board.promote_pawn(8, 0, kind));
        assert_eq!(board.piece_at(8), piece(Color::White, PieceKind::Pawn));
    }
}

#[test]
fn direct_stored_replies_apply_normal_moves_and_promotions() {
    let mut board = Board::default();
    assert!(board.apply_uci_move(parse_uci_move("f7f8").expect("valid UCI")));
    assert_eq!(board.piece_at(5), piece(Color::Black, PieceKind::Pawn));

    let mut promotion = Board::from_fen("k7/8/8/8/8/8/p7/7K b - - 0 1").expect("valid FEN");
    assert!(promotion.apply_uci_move(parse_uci_move("a2a1q").expect("valid UCI")));
    assert_eq!(
        promotion.piece_at(56),
        piece(Color::Black, PieceKind::Queen)
    );
}

#[test]
fn square_and_uci_round_trips_match_reference_indexing() {
    for index in 0..64 {
        let algebraic = square_to_algebraic(index).expect("board square");
        assert_eq!(algebraic_to_square(&algebraic), Some(index));
    }
    assert_eq!(square_to_algebraic(0).as_deref(), Some("a8"));
    assert_eq!(square_to_algebraic(63).as_deref(), Some("h1"));
    assert_eq!(square_to_algebraic(64), None);
    assert_eq!(algebraic_to_square("A1"), None);

    for notation in ["e2e4", "e1g1", "a7a8q", "b2b1n"] {
        let movement = parse_uci_move(notation).expect("valid UCI");
        assert_eq!(movement.to_string(), notation);
        assert_eq!(notation.parse::<UciMove>().expect("FromStr UCI"), movement);
    }
}

#[test]
fn invalid_uci_is_rejected_without_panicking_on_unicode() {
    for invalid in ["", "e2e", "e2e2", "i2e4", "e0e4", "a7a8k", "A7A8Q", "aé4"] {
        assert!(
            parse_uci_move(invalid).is_err(),
            "unexpectedly accepted: {invalid}"
        );
    }
}

#[test]
fn reference_v1_fixtures_parse_with_expected_ids_fens_and_solutions() {
    let collection = parse_puzzle_file(PUZZLES).expect("reference puzzles parse");
    assert_eq!(collection.version(), 1);
    assert_eq!(collection.title.as_deref(), Some("Lichess sample puzzles"));
    assert_eq!(collection.puzzles.len(), 2);
    assert_eq!(collection.puzzles[0].id, "lichess-001cr");
    assert_eq!(solution(&collection.puzzles[0]), vec!["d7e8"]);
    assert_eq!(collection.puzzles[1].id, "lichess-000hf");
    assert_eq!(
        solution(&collection.puzzles[1]),
        vec!["e2e6", "f7f8", "e6f7"]
    );

    let promotions = parse_puzzle_file(PROMOTIONS).expect("promotion puzzles parse");
    assert_eq!(promotions.puzzles.len(), 4);
    assert_eq!(solution(&promotions.puzzles[0]), vec!["a7a8q"]);
    assert_eq!(solution(&promotions.puzzles[1]), vec!["b7b8n"]);
    assert_eq!(solution(&promotions.puzzles[2]), vec!["h2h1r"]);
    assert_eq!(
        solution(&promotions.puzzles[3]),
        vec!["h2h3", "a2a1q", "h3h8"]
    );
}

#[test]
fn collection_title_description_difficulty_and_side_to_move_match_v1() {
    let collection = parse_puzzle_file(ENDGAMES).expect("endgame fixture parses");
    assert_eq!(collection.title.as_deref(), Some("Endgames"));
    assert_eq!(
        collection.puzzles[0].description.as_deref(),
        Some("Endgame one.")
    );
    assert_eq!(
        collection.puzzles[0]
            .difficulty
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("Easy")
    );
    assert_eq!(collection.puzzles[0].side_to_move(), Color::White);
    assert_eq!(
        collection.puzzles[1]
            .difficulty
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("2.5")
    );
    assert_eq!(collection.puzzles[1].side_to_move(), Color::Black);

    for (difficulty, expected) in [
        (json!("Hard"), Some("Hard")),
        (json!(4), Some("4")),
        (json!(-1), Some("-1")),
        (json!(2.5), Some("2.5")),
        (json!(null), None),
    ] {
        let mut value = sample_collection();
        value["puzzles"][0]["difficulty"] = difficulty;
        let parsed = parse_json(value).expect("difficulty accepted");
        assert_eq!(
            parsed.puzzles[0]
                .difficulty
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            expected
        );
    }
}

#[test]
fn puzzle_parser_rejects_duplicate_blank_ids_future_versions_and_bad_solutions() {
    let mut blank = sample_collection();
    blank["puzzles"][0]["id"] = json!("   ");
    assert!(parse_json(blank).is_err());

    let mut duplicate = sample_collection();
    let second = duplicate["puzzles"][0].clone();
    duplicate["puzzles"]
        .as_array_mut()
        .expect("puzzle array")
        .push(second);
    assert!(parse_json(duplicate).is_err());

    let mut future = sample_collection();
    future["version"] = json!(2);
    assert!(parse_json(future)
        .expect_err("future version rejected")
        .contains("version 2"));

    for solution in [
        json!([]),
        json!(["Qg7#"]),
        json!(["e2e2"]),
        json!(["a7a8k"]),
    ] {
        let mut value = sample_collection();
        value["puzzles"][0]["solution"] = solution;
        assert!(parse_json(value).is_err());
    }
}

#[test]
fn puzzle_parser_rejects_invalid_fen_empty_collection_and_oversize_file() {
    let mut invalid_fen = sample_collection();
    invalid_fen["puzzles"][0]["fen"] = json!("invalid");
    assert!(parse_json(invalid_fen)
        .expect_err("bad FEN rejected")
        .contains("FEN"));
    assert!(parse_json(json!({"version": 1, "puzzles": []})).is_err());
    assert!(parse_puzzle_file(&vec![b' '; MAX_PUZZLE_FILE_BYTES + 1]).is_err());
}

#[test]
fn collection_filename_filtering_is_sorted_deduplicated_and_platform_neutral() {
    let names = sorted_puzzle_collection_filenames([
        "notes.json",
        "puzzles-z.json",
        "puzzles-.json",
        "puzzles.json",
        "puzzles-a.json",
        "puzzles-a.json",
        "dir/puzzles-b.json",
        "puzzles-c.JSON",
    ]);
    assert_eq!(names, ["puzzles-a.json", "puzzles-z.json", "puzzles.json"]);
}

#[test]
fn progress_round_trips_ids_deduplicates_solved_and_uses_puzzle_ids() {
    let parsed = Progress::parse(PROGRESS).expect("v1 progress parses");
    assert_eq!(parsed.version(), 1);
    assert_eq!(parsed.active_file.as_deref(), Some("puzzles-endgames.json"));
    let file = parsed.file("puzzles-endgames.json").expect("file progress");
    assert_eq!(file.current_puzzle_id.as_deref(), Some("end-2"));
    assert_eq!(
        file.solved_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["end-1", "end-2"]
    );

    let bytes = parsed.to_bytes().expect("progress serializes");
    assert_eq!(Progress::parse(&bytes).expect("round trip"), parsed);

    let mut progress = Progress::new();
    assert!(progress.set_active_file("puzzles.json"));
    assert!(progress.remember_puzzle("puzzles.json", "lichess-000hf"));
    assert!(progress.mark_solved("puzzles.json", "lichess-001cr"));
    assert!(!progress.mark_solved("puzzles.json", "lichess-001cr"));
    assert!(progress.is_solved("puzzles.json", "lichess-001cr"));
}

#[test]
fn progress_rejects_corrupt_and_future_versions() {
    assert!(Progress::parse(b"not json").is_err());
    let future = br#"{"version":2,"active_file":null,"files":{}}"#;
    assert!(Progress::parse(future)
        .expect_err("future progress rejected")
        .contains("Unsupported progress version 2"));
}

#[test]
fn fen_retains_validated_fullmove_number() {
    for (color, number) in [("w", 1), ("b", 37), ("w", u32::MAX)] {
        let position = parse_fen(&format!("8/8/8/8/8/8/8/8 {color} - - 0 {number}")).unwrap();
        assert_eq!(position.fullmove_number(), number);
    }
}
