use chess_core::play::{PlayApp, Request, Response, Snapshot};
use chess_render::{render, DisplayMetrics, HitTarget, PlayLayout, PlayTarget, Rect};
fn metrics(width: u32, height: u32) -> DisplayMetrics {
    DisplayMetrics {
        width,
        height,
        dpi: 300,
    }
}
fn ready() -> PlayApp {
    let mut app = PlayApp::default();
    let id = app.begin(Request::Current).unwrap();
    let value = serde_json::json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":chess_core::play::START_FEN,"moves":[],"ply":0,"status":"playing","result":null,"lastMove":null});
    app.complete(id, Ok(Response::Game(Snapshot::parse(&value).unwrap())));
    app
}
#[test]
fn play_layout_maps_only_play_controls_and_white_squares() {
    for metrics in [metrics(1860, 2480), metrics(1404, 1872)] {
        let app = ready();
        let layout = PlayLayout::new(metrics).unwrap();
        let output = render(&app, metrics).unwrap();
        for sq in 0..64 {
            let r = layout.base.square_rect(sq);
            assert_eq!(
                output.hit_test_app(r.x + r.width / 2, r.y + r.height / 2, &app),
                Some(HitTarget::Play(PlayTarget::Square(sq)))
            );
        }
        for (target, rect, enabled) in layout.controls(&app) {
            assert!(
                rect.width >= 118 && rect.height >= 118,
                "{target:?}: {rect:?}"
            );
            assert!(!rect.intersects(layout.base.board));
            if !enabled {
                assert!(output
                    .hit_test_app(rect.x + rect.width / 2, rect.y + rect.height / 2, &app)
                    .is_none());
                continue;
            }
            assert_eq!(
                output.hit_test_app(rect.x + rect.width / 2, rect.y + rect.height / 2, &app),
                Some(HitTarget::Play(target))
            );
        }
    }
}
#[test]
fn waiting_and_finished_block_board_but_close_and_refresh_survive() {
    let metrics = metrics(1404, 1872);
    let mut app = ready();
    let layout = PlayLayout::new(metrics).unwrap();
    let r = layout.base.square_rect(52);
    app.begin(Request::Move {
        uci: "e2e4".into(),
        expected_ply: 0,
    });
    let output = render(&app, metrics).unwrap();
    assert!(output.hit_test_app(r.x + 5, r.y + 5, &app).is_none());
    for target in [PlayTarget::Close, PlayTarget::FullRefresh] {
        let (_, rect, _) = layout
            .controls(&app)
            .into_iter()
            .find(|c| c.0 == target)
            .unwrap();
        assert_eq!(
            output.hit_test_app(rect.x + 10, rect.y + 10, &app),
            Some(HitTarget::Play(target))
        );
    }
}
#[test]
fn frames_have_stable_snapshots_and_damage_reconstructs() {
    let metrics = metrics(1404, 1872);
    let mut app = ready();
    let mut frames = vec![];
    let id = app.begin(Request::Current).unwrap();
    app.complete(id, Err("Configure serverUrl in config.json".into()));
    frames.push(("disconnected", render(&app, metrics).unwrap().frame));
    app = PlayApp::default();
    frames.push(("ready", render(&app, metrics).unwrap().frame));
    app = ready();
    frames.push(("playing", render(&app, metrics).unwrap().frame));
    let id = app
        .begin(Request::Move {
            uci: "e2e4".into(),
            expected_ply: 0,
        })
        .unwrap();
    frames.push(("waiting", render(&app, metrics).unwrap().frame));
    app.complete(
        id,
        Ok(Response::Error {
            status: 422,
            code: "ILLEGAL_MOVE".into(),
            message: "bad".into(),
            state: None,
        }),
    );
    frames.push(("illegal", render(&app, metrics).unwrap().frame));
    let mut snapshot = app.snapshot.clone().unwrap();
    snapshot.status = "finished".into();
    snapshot.result = Some(chess_core::play::GameResult {
        outcome: "black_win".into(),
        reason: "resignation".into(),
    });
    let id = app.begin(Request::Resign).unwrap();
    app.complete(id, Ok(Response::Game(snapshot)));
    frames.push(("finished", render(&app, metrics).unwrap().frame));
    let mut hashes = vec![];
    for (name, frame) in &frames {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in frame.pixels() {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
        hashes.push(format!("{name} {hash:016x}"));
        let mut bytes = format!("P5\n{} {}\n255\n", metrics.width, metrics.height).into_bytes();
        bytes.extend_from_slice(frame.pixels());
        std::fs::write(format!("../../target/play-{name}.pgm"), bytes).unwrap();
    }
    let actual = hashes.join("\n") + "\n";
    assert_eq!(
        actual,
        include_str!("../../../tests/fixtures/play/snapshots.txt")
    );
    for pair in frames.windows(2) {
        let damage = chess_render::calculate_damage(Some(&pair[0].1), &pair[1].1);
        assert!(!damage.is_empty());
        let mut replay = pair[0].1.clone();
        for rect in damage {
            for y in rect.y..rect.bottom() {
                for x in rect.x..rect.right() {
                    replay.fill_rect(
                        Rect::new(x, y, 1, 1),
                        pair[1].1.pixels()[(y * metrics.width + x) as usize],
                    );
                }
            }
        }
        assert_eq!(replay, pair[1].1);
    }
}
#[test]
fn promotion_modal_is_exclusive_and_finished_board_is_read_only() {
    let metrics = metrics(1404, 1872);
    let mut app = ready();
    app.promotion = Some((8, 0));
    let layout = PlayLayout::new(metrics).unwrap();
    let output = render(&app, metrics).unwrap();
    for (i, kind) in [
        chess_core::PieceKind::Queen,
        chess_core::PieceKind::Rook,
        chess_core::PieceKind::Bishop,
        chess_core::PieceKind::Knight,
    ]
    .into_iter()
    .enumerate()
    {
        let r = layout.base.promotion_choices[i];
        assert_eq!(
            output.hit_test_app(r.x + r.width / 2, r.y + r.height / 2, &app),
            Some(HitTarget::Play(PlayTarget::Promotion(kind)))
        );
    }
    app.cancel_selection();
    let id = app.begin(Request::Resign).unwrap();
    let mut snapshot = app.snapshot.clone().unwrap();
    snapshot.status = "finished".into();
    snapshot.result = Some(chess_core::play::GameResult {
        outcome: "black_win".into(),
        reason: "resignation".into(),
    });
    app.complete(id, Ok(Response::Game(snapshot)));
    let output = render(&app, metrics).unwrap();
    let r = layout.base.square_rect(52);
    assert!(output.hit_test_app(r.x + 5, r.y + 5, &app).is_none());
}

#[test]
fn close_icon_matches_existing_products_and_keeps_its_touch_rectangle() {
    use chess_core::{parse_puzzle_file, ActiveCollection, AppState, Progress, WorkspaceSettings};
    let puzzles = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let review = chess_core::review_app::ReviewApp::new(vec![], WorkspaceSettings::default());
    for metrics in [metrics(1860, 2480), metrics(1404, 1872)] {
        let app = PlayApp::default();
        let play = render(&app, metrics).unwrap();
        let exit = play.layout.exit;
        let visual = exit.inset(play.layout.header.height / 10);
        for reference in [
            render(&puzzles, metrics).unwrap(),
            render(&review, metrics).unwrap(),
        ] {
            assert_eq!(exit, reference.layout.exit);
            for y in visual.y..visual.bottom() {
                for x in visual.x..visual.right() {
                    assert_eq!(play.frame.pixel(x,y), reference.frame.pixel(x,y),
                        "Play close must contain the same X and chrome, without CLOSE text, at {x},{y} for {metrics:?}");
                }
            }
        }
        let icon = visual.inset(visual.width.min(visual.height) / 4);
        for (x, y) in [
            (icon.x, icon.y),
            (icon.right() - 1, icon.y),
            (icon.x, icon.bottom() - 1),
            (icon.right() - 1, icon.bottom() - 1),
        ] {
            assert_eq!(play.frame.pixel(x, y), Some(0), "X diagonal endpoint");
        }
        for (x, y) in [
            (exit.x, exit.y),
            (exit.right() - 1, exit.bottom() - 1),
            (exit.x + exit.width / 2, exit.y + exit.height / 2),
        ] {
            assert_eq!(
                play.hit_test_app(x, y, &app),
                Some(HitTarget::Play(PlayTarget::Close))
            );
        }
        for (x, y) in [
            (exit.x - 1, exit.y + exit.height / 2),
            (exit.x + exit.width / 2, exit.bottom()),
        ] {
            assert_eq!(play.hit_test_app(x, y, &app), None);
        }
    }
}

#[test]
fn black_orientation_maps_all_squares_and_color_controls_are_guarded() {
    for metrics in [metrics(1860, 2480), metrics(1404, 1872)] {
        let mut app = PlayApp::default();
        app.choose_color(chess_core::Color::Black);
        let value = serde_json::json!({"apiVersion":1,"rating":1500,"playerColor":"black","fen":"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1","moves":["e2e4"],"ply":1,"status":"playing","result":null,"lastMove":"e2e4"});
        let id = app.begin(Request::Current).unwrap();
        app.complete(id, Ok(Response::Game(Snapshot::parse(&value).unwrap())));
        let frame = render(&app, metrics).unwrap();
        std::fs::write(
            format!("../../target/play-black-{}.pgm", metrics.width),
            frame.frame.to_pgm(),
        )
        .unwrap();
        for displayed in 0..64 {
            let r = frame.layout.square_rect(displayed);
            assert_eq!(
                frame.hit_test_app(r.x + r.width / 2, r.y + r.height / 2, &app),
                Some(HitTarget::Play(PlayTarget::Square(63 - displayed)))
            );
        }
        for (target, rect, enabled) in PlayLayout::new(metrics).unwrap().controls(&app) {
            if matches!(target, PlayTarget::ChooseColor(_)) {
                assert!(!enabled);
                assert_eq!(
                    frame.hit_test_app(rect.x + rect.width / 2, rect.y + rect.height / 2, &app),
                    None
                );
            }
        }
        assert!(chess_render::ScreenState::flipped(&app));
    }
}
