use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, Board, Color,
    PieceKind, Progress, PromotionChoice, SolutionFeedback,
};
use serde::Deserialize;

const PARITY_PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const PARITY_VECTORS: &[u8] = include_bytes!("../../../tests/fixtures/parity-vectors.json");

#[derive(Debug, Deserialize)]
struct ParityVector {
    id: String,
    start_fen: String,
    side_to_move: String,
    solver_actions: Vec<String>,
    automatic_replies: Vec<String>,
    final_placement: String,
    complete: bool,
    solved: bool,
}

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn choose_promotion(suffix: u8) -> PromotionChoice {
    match suffix {
        b'q' => PromotionChoice::Queen,
        b'r' => PromotionChoice::Rook,
        b'b' => PromotionChoice::Bishop,
        b'n' => PromotionChoice::Knight,
        _ => panic!("invalid promotion suffix"),
    }
}

fn play(app: &mut AppState, movement: &str) {
    app.dispatch(Action::TapSquare(square(&movement[..2])));
    app.dispatch(Action::TapSquare(square(&movement[2..4])));
    if movement.len() == 5 {
        app.dispatch(Action::ChoosePromotion(choose_promotion(
            movement.as_bytes()[4],
        )));
    }
}

fn placement(board: &Board) -> String {
    let mut rows = Vec::with_capacity(8);
    for rank in 0..8 {
        let mut row = String::new();
        let mut empty = 0usize;
        for file in 0..8 {
            let square = rank * 8 + file;
            if let Some(piece) = board.piece_at(square) {
                if empty > 0 {
                    row.push_str(&empty.to_string());
                    empty = 0;
                }
                let symbol = match piece.kind {
                    PieceKind::Pawn => 'p',
                    PieceKind::Knight => 'n',
                    PieceKind::Bishop => 'b',
                    PieceKind::Rook => 'r',
                    PieceKind::Queen => 'q',
                    PieceKind::King => 'k',
                };
                row.push(match piece.color {
                    Color::White => symbol.to_ascii_uppercase(),
                    Color::Black => symbol,
                });
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            row.push_str(&empty.to_string());
        }
        rows.push(row);
    }
    rows.join("/")
}

#[test]
fn shared_reference_vectors_match_start_line_replies_final_board_and_solved_state() {
    let collection = parse_puzzle_file(PARITY_PUZZLES).expect("parity puzzles parse");
    let vectors: Vec<ParityVector> =
        serde_json::from_slice(PARITY_VECTORS).expect("parity vectors parse");

    assert_eq!(vectors.len(), collection.puzzles.len());

    for vector in vectors {
        let puzzle_index = collection
            .puzzles
            .iter()
            .position(|puzzle| puzzle.id == vector.id)
            .expect("vector puzzle exists");
        let puzzle = &collection.puzzles[puzzle_index];

        assert_eq!(puzzle.fen, vector.start_fen, "{} start FEN", vector.id);
        assert_eq!(
            puzzle.side_to_move(),
            if vector.side_to_move == "b" {
                Color::Black
            } else {
                Color::White
            },
            "{} side to move",
            vector.id
        );
        assert_eq!(
            puzzle
                .solution
                .iter()
                .step_by(2)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vector
                .solver_actions
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            "{} solver actions",
            vector.id
        );
        assert_eq!(
            puzzle
                .solution
                .iter()
                .skip(1)
                .step_by(2)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vector
                .automatic_replies
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            "{} automatic replies",
            vector.id
        );

        let mut app = AppState::new(
            ActiveCollection::from_collection("parity-puzzles.json", collection.clone()),
            Progress::new(),
        );
        for _ in 0..puzzle_index {
            app.dispatch(Action::NextPuzzle);
        }

        assert_eq!(app.active_puzzle().id, vector.id);
        assert_eq!(app.flipped(), vector.side_to_move == "b");

        for movement in &vector.solver_actions {
            play(&mut app, movement);
        }

        assert_eq!(
            placement(app.board()),
            vector.final_placement,
            "{} final board",
            vector.id
        );
        assert_eq!(
            app.feedback() == SolutionFeedback::Complete,
            vector.complete,
            "{} completion",
            vector.id
        );
        assert_eq!(
            app.is_current_solved(),
            vector.solved,
            "{} solved state",
            vector.id
        );
    }
}

#[test]
fn application_grades_all_four_promotion_choices_exactly() {
    for (suffix, choice, expected_kind) in [
        ('q', PromotionChoice::Queen, PieceKind::Queen),
        ('r', PromotionChoice::Rook, PieceKind::Rook),
        ('b', PromotionChoice::Bishop, PieceKind::Bishop),
        ('n', PromotionChoice::Knight, PieceKind::Knight),
    ] {
        let collection = format!(
            r#"{{"version":1,"puzzles":[{{"id":"promotion-{suffix}","fen":"7k/P7/8/8/8/8/8/K7 w - - 0 1","solution":["a7a8{suffix}"]}}]}}"#
        );
        let parsed = parse_puzzle_file(collection.as_bytes()).expect("promotion fixture parses");
        let mut app = AppState::new(
            ActiveCollection::from_collection("promotions.json", parsed),
            Progress::new(),
        );

        app.dispatch(Action::TapSquare(square("a7")));
        app.dispatch(Action::TapSquare(square("a8")));
        assert!(app.pending_promotion().is_some());

        app.dispatch(Action::ChoosePromotion(choice));
        assert_eq!(app.feedback(), SolutionFeedback::Complete);
        assert!(app.is_current_solved());
        assert_eq!(
            app.board().piece_at(square("a8")).map(|piece| piece.kind),
            Some(expected_kind)
        );
    }
}
