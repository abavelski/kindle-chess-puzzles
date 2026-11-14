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
    assert!(app
        .begin(Request::New {
            rating: 1500,
            player_color: chess_core::Color::White
        })
        .is_some());
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

fn black_opening() -> Snapshot {
    Snapshot::parse(&json!({"apiVersion":1,"rating":1500,"playerColor":"black","fen":"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1","moves":["e2e4"],"ply":1,"status":"playing","result":null,"lastMove":"e2e4"})).unwrap()
}
#[test]
fn black_creation_reconnect_selection_and_turn_parity() {
    let mut app = PlayApp::default();
    app.choose_color(chess_model::Color::Black);
    let request = Request::New {
        rating: 1500,
        player_color: chess_model::Color::Black,
    };
    let (_, _, body) = request.wire();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).unwrap(),
        json!({"rating":1500,"playerColor":"black"})
    );
    let id = app.begin(request).unwrap();
    app.complete(id, Ok(Response::Game(black_opening())));
    assert!(app.flipped());
    assert_eq!(app.message, "Black to move");
    assert_eq!(app.snapshot.as_ref().unwrap().ply, 1);
    app.choose_color(chess_model::Color::White);
    assert!(app.flipped());
    app.tap(sq("e4").unwrap());
    assert!(app.selected.is_none());
    app.tap(sq("e7").unwrap());
    assert_eq!(
        app.tap(sq("e5").unwrap()),
        Some(Request::Move {
            uci: "e7e5".into(),
            expected_ply: 1
        })
    );
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::Game(initial())));
    assert!(!app.flipped());
    let mut value = serde_json::to_value(black_opening()).unwrap();
    value["playerColor"] = json!("white");
    assert!(Snapshot::parse(&value).is_err());
    value["status"] = json!("finished");
    value["result"] = json!({"outcome":"white_win","reason":"resignation"});
    assert!(Snapshot::parse(&value).is_err()); // White cannot resign to a White win.
    value["playerColor"] = json!("black");
    assert!(Snapshot::parse(&value).is_ok());
}
#[test]
fn black_promotion_and_legacy_response_are_safe() {
    let mut app = PlayApp::default();
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::Game(black_opening())));
    app.board = chess_model::Board::from_fen("4k3/8/8/8/8/8/p7/4K3 b - - 0 1").unwrap();
    for kind in [
        chess_model::PieceKind::Queen,
        chess_model::PieceKind::Rook,
        chess_model::PieceKind::Bishop,
        chess_model::PieceKind::Knight,
    ] {
        app.tap(sq("a2").unwrap());
        app.tap(sq("a1").unwrap());
        assert!(app.promotion.is_some());
        let before = app.board.clone();
        let request = app.promote(kind).unwrap();
        assert_eq!(
            request,
            Request::Move {
                uci: format!(
                    "a2a1{}",
                    match kind {
                        chess_model::PieceKind::Queen => 'q',
                        chess_model::PieceKind::Rook => 'r',
                        chess_model::PieceKind::Bishop => 'b',
                        _ => 'n',
                    }
                ),
                expected_ply: 1
            }
        );
        assert_eq!(app.board, before);
    }
    let request = Request::New {
        rating: 1500,
        player_color: chess_model::Color::Black,
    };
    assert!(chess_core::play::parse_response(
        &request,
        201,
        &serde_json::to_vec(&initial()).unwrap()
    )
    .unwrap_err()
    .contains("updated desktop"));
}

#[test]
fn history_is_read_only_latest_restores_exact_server_state_and_san_is_additive() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/play/castle.json")).unwrap();
    value["sanMoves"] = json!(["Nf3", "Nf6", "e3", "e6", "Be2", "Be7", "O-O", "O-O"]);
    value["gameId"] = json!("fixture-game");
    value["startedAt"] = json!("2026-11-09T12:00:00Z");
    value["lastMoveAt"] = json!("2026-11-09T12:01:00Z");
    let state = Snapshot::parse(&value).unwrap();
    assert_eq!(state.move_label(7), Some("O-O"));
    let positions = chess_core::play::replay_positions(&state.moves).unwrap();
    assert_eq!(positions.len(), state.ply + 1);
    let mut app = PlayApp::default();
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::Game(state.clone())));
    for ply in [0, 1, 4, 7] {
        assert!(app.view_history(ply));
        assert_eq!(app.board.placement(), positions[ply].placement());
        assert!(app.viewing_history());
        assert!(!app.can_move());
        assert!(app.begin(Request::Resign).is_none());
        assert!(app.tap(sq("e2").unwrap()).is_none());
        assert_eq!(app.snapshot.as_ref().unwrap(), &state);
    }
    app.latest();
    assert!(!app.viewing_history());
    assert_eq!(app.view_ply, state.ply);
    assert_eq!(app.board, chess_model::Board::from_fen(&state.fen).unwrap());
    assert!(app.can_move());
    app.view_history(0);
    let id = app.begin(Request::Current).unwrap();
    assert!(!app.view_history(1));
    app.complete(id, Ok(Response::Game(state)));
    assert!(!app.viewing_history());
    value["sanMoves"] = json!(["Nf3"]);
    assert!(Snapshot::parse(&value).is_err());
    value.as_object_mut().unwrap().remove("sanMoves");
    assert_eq!(Snapshot::parse(&value).unwrap().move_label(7), Some("e1g1"));
}
#[test]
fn replay_positions_cover_all_special_moves_and_finished_history() {
    for fixture in [
        include_str!("../../../tests/fixtures/play/castle.json"),
        include_str!("../../../tests/fixtures/play/en-passant.json"),
        include_str!("../../../tests/fixtures/play/promotion-q.json"),
        include_str!("../../../tests/fixtures/play/promotion-r.json"),
        include_str!("../../../tests/fixtures/play/promotion-b.json"),
        include_str!("../../../tests/fixtures/play/promotion-n.json"),
    ] {
        let mut value: serde_json::Value = serde_json::from_str(fixture).unwrap();
        value["status"] = json!("finished");
        value["result"] = json!({"outcome":"draw","reason":"draw"});
        let state = Snapshot::parse(&value).unwrap();
        let positions = chess_core::play::replay_positions(&state.moves).unwrap();
        assert_eq!(
            positions.last().unwrap().placement(),
            state.fen.split_whitespace().next().unwrap()
        );
        let mut app = PlayApp::default();
        let id = app.begin(Request::Current).unwrap();
        app.complete(id, Ok(Response::Game(state.clone())));
        for (ply, position) in positions.iter().enumerate() {
            assert!(app.view_history(ply));
            assert_eq!(app.board.placement(), position.placement());
            assert!(!app.can_move());
        }
    }
}

#[test]
fn unsupported_black_start_is_actionable_and_does_not_commit() {
    let mut app = PlayApp::default();
    app.choose_color(chess_model::Color::Black);
    let before = app.board.clone();
    let id = app.begin(app.new_game_request()).unwrap();
    app.complete(
        id,
        Ok(Response::Error {
            status: 400,
            code: "BAD_REQUEST".into(),
            message: "unknown field".into(),
            state: None,
        }),
    );
    assert!(app.message.contains("updated desktop"));
    assert_eq!(app.board, before);
    assert!(app.snapshot.is_none());
    assert!(!app.can_new());
}

#[test]
fn white_legacy_fallback_only_follows_confirmed_validation_rejection() {
    let mut app = PlayApp::default();
    let id = app.begin(app.new_game_request()).unwrap();
    let next = app
        .complete(
            id,
            Ok(Response::Error {
                status: 400,
                code: "BAD_REQUEST".into(),
                message: "rating only".into(),
                state: None,
            }),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&next.wire().2).unwrap(),
        json!({"rating":1500})
    );
    let mut success = app.clone();
    let success_id = success.begin(next.clone()).unwrap();
    success.complete(success_id, Ok(Response::Game(initial())));
    assert!(success.can_move());
    let id = app.begin(next).unwrap();
    assert!(app
        .complete(
            id,
            Ok(Response::Error {
                status: 400,
                code: "BAD_REQUEST".into(),
                message: "still bad".into(),
                state: None
            })
        )
        .is_none());
    let mut app = PlayApp::default();
    let id = app.begin(app.new_game_request()).unwrap();
    assert!(app.complete(id, Err("Timeout".into())).is_none());
    assert!(!app.can_new());
}

#[test]
fn selected_human_color_rotates_finished_game_without_mutating_server_snapshot() {
    let mut state = initial();
    state.status = "finished".into();
    state.result = Some(chess_core::play::GameResult {
        outcome: "black_win".into(),
        reason: "resignation".into(),
    });
    let mut app = PlayApp::default();
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Ok(Response::Game(state.clone())));
    for color in [
        chess_model::Color::Black,
        chess_model::Color::White,
        chess_model::Color::Black,
    ] {
        app.choose_color(color);
        assert_eq!(app.flipped(), color == chess_model::Color::Black);
        assert_eq!(app.snapshot.as_ref(), Some(&state));
        assert_eq!(
            app.new_game_request(),
            Request::New {
                rating: 1500,
                player_color: color
            }
        );
    }
    let id = app.begin(app.new_game_request()).unwrap();
    assert!(app.flipped());
    app.complete(id, Ok(Response::Game(black_opening())));
    app.choose_color(chess_model::Color::White);
    assert!(app.flipped());
}
