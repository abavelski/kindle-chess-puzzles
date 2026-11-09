use chess_core::play::{PlayApp, Request, Response, Snapshot, UiStatus};
use chess_model::algebraic_to_square as sq;
use serde_json::json;
fn initial() -> Snapshot {
    Snapshot::parse(&json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1","moves":[],"ply":0,"status":"playing","result":null,"lastMove":null})).unwrap()
}
fn playing() -> PlayApp {
    let mut app = PlayApp::default();
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::Game(initial())));
    app
}
#[test]
fn taps_never_commit_and_duplicate_requests_are_suppressed() {
    let mut app = playing();
    let before = app.board.clone();
    assert!(app.tap(sq("e2").unwrap()).is_none());
    let request = app.tap(sq("e4").unwrap()).unwrap();
    assert_eq!(
        request,
        Request::Move {
            uci: "e2e4".into(),
            expected_ply: 0
        }
    );
    let id = app.begin(request).unwrap();
    assert_eq!(app.status, UiStatus::Waiting);
    assert!(app.tap(sq("d2").unwrap()).is_none());
    assert!(app.begin(Request::Resign).is_none());
    assert_eq!(app.board, before);
    app.complete(id, Err("Connection lost".into()));
    assert_eq!(app.board, before);
    assert!(app.message.contains("Refresh"));
    assert!(app.tap(sq("e2").unwrap()).is_none());
    assert!(app.selected.is_none());
    assert!(app.begin(Request::Current).is_some());
}
#[test]
fn illegal_stale_finished_and_late_responses() {
    let mut app = playing();
    let before = app.board.clone();
    let id = app
        .begin(Request::Move {
            uci: "e2e5".into(),
            expected_ply: 0,
        })
        .unwrap();
    app.complete(
        id,
        Ok(Response::Error {
            status: 422,
            code: "ILLEGAL_MOVE".into(),
            message: "bad".into(),
            state: Some(initial()),
        }),
    );
    assert_eq!(app.board, before);
    assert_eq!(app.message, "Illegal move");
    let id = app.begin(Request::Current).unwrap();
    app.complete(id.wrapping_sub(1), Ok(Response::NoGame));
    assert!(app.in_flight());
    app.complete(
        id,
        Ok(Response::Error {
            status: 409,
            code: "STALE_PLY".into(),
            message: "stale".into(),
            state: Some(initial()),
        }),
    );
    assert!(app.message.contains("select"));
    let mut snapshot = initial();
    snapshot.status = "finished".into();
    snapshot.result = Some(chess_core::play::GameResult {
        outcome: "black_win".into(),
        reason: "resignation".into(),
    });
    let id = app.begin(Request::Resign).unwrap();
    app.complete(id, Ok(Response::Game(snapshot)));
    assert_eq!(app.status, UiStatus::Finished);
    assert!(app.tap(sq("e2").unwrap()).is_none());
    assert!(app.begin(Request::New { rating: 1500 }).is_some());
}
#[test]
fn ratings_health_reconnect_and_contract_validation() {
    let mut app = PlayApp::default();
    for _ in 0..30 {
        app.change_rating(-100);
    }
    assert_eq!(app.rating, 600);
    for _ in 0..30 {
        app.change_rating(100);
    }
    assert_eq!(app.rating, 2600);
    let id = app.begin(Request::Health).unwrap();
    assert_eq!(
        app.complete(id, Ok(Response::Health(true))),
        Some(Request::Current)
    );
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::NoGame));
    assert_eq!(app.status, UiStatus::Idle);
    let mut value = json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":initial().fen,"moves":[],"ply":0,"status":"playing","result":null,"lastMove":null});
    for (key, bad) in [
        ("ply", json!(2)),
        ("rating", json!(650)),
        ("playerColor", json!("black")),
        ("apiVersion", json!(2)),
        ("fen", json!("8/8/8/8/8/8/8/8 w - - 0 1")),
        ("lastMove", json!("e2e4")),
    ] {
        let old = value[key].clone();
        value[key] = bad;
        assert!(Snapshot::parse(&value).is_err(), "{key}");
        value[key] = old;
    }
}
#[test]
fn castling_and_all_promotions_preserve_server_positions() {
    for fixture in [
        include_str!("../../../tests/fixtures/play/castle.json"),
        include_str!("../../../tests/fixtures/play/promotion-q.json"),
        include_str!("../../../tests/fixtures/play/promotion-r.json"),
        include_str!("../../../tests/fixtures/play/promotion-b.json"),
        include_str!("../../../tests/fixtures/play/promotion-n.json"),
        include_str!("../../../tests/fixtures/play/en-passant.json"),
    ] {
        let mut value: serde_json::Value = serde_json::from_str(fixture).unwrap();
        Snapshot::parse(&value).unwrap();
        value["moves"][0] = json!("e2e3");
        assert!(Snapshot::parse(&value).is_err());
    }
    for kind in [
        chess_model::PieceKind::Queen,
        chess_model::PieceKind::Rook,
        chess_model::PieceKind::Bishop,
        chess_model::PieceKind::Knight,
    ] {
        let mut app = playing();
        app.board = chess_model::Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        app.tap(sq("a7").unwrap());
        assert!(app.tap(sq("a8").unwrap()).is_none());
        assert!(app.promotion.is_some());
        let before = app.board.clone();
        assert!(matches!(app.promote(kind), Some(Request::Move { .. })));
        assert_eq!(app.board, before);
    }
}
#[test]
fn confirmed_white_and_black_moves_replace_snapshot_atomically() {
    let mut app = playing();
    let id = app
        .begin(Request::Move {
            uci: "e2e4".into(),
            expected_ply: 0,
        })
        .unwrap();
    let value = json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2","moves":["e2e4","e7e5"],"ply":2,"status":"playing","result":null,"lastMove":"e7e5"});
    app.complete(id, Ok(Response::Game(Snapshot::parse(&value).unwrap())));
    assert_eq!(app.snapshot.as_ref().unwrap().ply, 2);
    assert!(app.board.piece_at(sq("e2").unwrap()).is_none());
    assert!(app.board.piece_at(sq("e5").unwrap()).is_some());
    assert!(app.can_move());
    let rating = app.rating;
    app.change_rating(100);
    assert_eq!(app.rating, rating);
}
#[test]
fn unavailable_engine_remains_visible_after_current_reports_no_game() {
    let mut app = PlayApp::default();
    let id = app.begin(Request::Health).unwrap();
    assert_eq!(
        app.complete(id, Ok(Response::Health(false))),
        Some(Request::Current)
    );
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::NoGame));
    assert!(app.message.contains("Maia unavailable"));
    let id = app.begin(Request::Current).unwrap();
    app.complete(
        id,
        Ok(Response::Error {
            status: 409,
            code: "BUSY".into(),
            message: "busy".into(),
            state: None,
        }),
    );
    assert!(app.message.contains("Refresh"));
    assert!(!app.can_new());
}
#[test]
fn finished_conflict_shows_result_and_keeps_board_disabled() {
    let mut app = playing();
    let mut state = initial();
    state.status = "finished".into();
    state.result = Some(chess_core::play::GameResult {
        outcome: "black_win".into(),
        reason: "resignation".into(),
    });
    let id = app.begin(Request::Current).unwrap();
    app.complete(
        id,
        Ok(Response::Error {
            status: 409,
            code: "GAME_FINISHED".into(),
            message: "finished".into(),
            state: Some(state),
        }),
    );
    assert_eq!(app.status, UiStatus::Finished);
    assert!(app.message.contains("resignation"));
    assert!(!app.can_move());
    assert!(app.can_new());
}
