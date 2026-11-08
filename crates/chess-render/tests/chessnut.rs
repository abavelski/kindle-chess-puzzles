use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, BoardSessionStatus as Status, Progress,
};
use chess_render::{render, DisplayMetrics, HitTarget};
fn app() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    )
}
const METRICS: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};
fn center(rect: chess_render::Rect) -> (u32, u32) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}
#[test]
fn send_is_inert_only_during_submission_then_ready_without_done() {
    for small in [false, true] {
        for metrics in [
            METRICS,
            DisplayMetrics {
                width: 2480,
                height: 1860,
                dpi: 300,
            },
        ] {
            let mut app = app();
            if small {
                app.dispatch(Action::ToggleBoardSizeSetting);
            }
            assert_eq!(
                render(&app, metrics).unwrap().layout.toolbar_targets[6]
                    .rect
                    .width,
                0
            );
            app.dispatch(Action::BoardStatus(Status::Connected));
            let ready = render(&app, metrics).unwrap();
            let send = ready.layout.toolbar_targets[6].rect;
            assert!(send.x >= ready.layout.toolbar_targets[0].rect.right());
            let (x, y) = center(send);
            assert_eq!(
                ready.hit_test_app(x, y, &app),
                Some(HitTarget::SendBoardPosition)
            );
            app.dispatch(Action::SendBoardPosition);
            for status in [Status::Sending] {
                app.dispatch(Action::BoardStatus(status.clone()));
                let busy = render(&app, metrics).unwrap();
                let send = busy.layout.toolbar_targets[6].rect;
                assert!(send.width >= busy.layout.minimum_touch_px());
                let (x, y) = center(send);
                assert_eq!(busy.hit_test_app(x, y, &app), None);
                let visual = busy.layout.control_visual_rect(send);
                assert_eq!(
                    busy.frame.pixel(visual.x + visual.width / 2, visual.y + 10),
                    Some(0),
                    "selected send has an inverted fill"
                );
                assert_eq!(busy.layout.toolbar_targets.len(), 7, "no DONE control");
                let (bx, by) = center(busy.layout.square_rect(0));
                assert_eq!(
                    busy.hit_test_app(bx, by, &app),
                    Some(HitTarget::Square(0)),
                    "no confirmation/pending modal"
                );
                let (sx, sy) = center(busy.layout.settings);
                assert_eq!(
                    busy.hit_test_app(sx, sy, &app),
                    Some(HitTarget::OpenSettings)
                );
                let controls: Vec<_> = busy
                    .layout
                    .toolbar_targets
                    .iter()
                    .filter(|c| c.rect.width > 0)
                    .collect();
                for (i, control) in controls.iter().enumerate() {
                    assert!(control.rect.height >= busy.layout.minimum_touch_px());
                    for other in &controls[i + 1..] {
                        assert!(!control.rect.intersects(other.rect));
                    }
                }
            }
            app.dispatch(Action::BoardStatus(Status::Connected));
            let ready = render(&app, metrics).unwrap();
            let (x, y) = center(ready.layout.toolbar_targets[6].rect);
            assert_eq!(
                ready.hit_test_app(x, y, &app),
                Some(HitTarget::SendBoardPosition)
            );
        }
    }
}
#[test]
fn settings_disconnect_control_and_review_remains_unchanged() {
    for review in [false, true] {
        let mut app = app();
        app.dispatch(Action::BoardStatus(Status::Connected));
        if review {
            let games = chess_core::parse_review_file(include_bytes!(
                "../../../tests/fixtures/game-review/valid-standard.json"
            ))
            .unwrap();
            app.set_review_games(vec![chess_core::ReviewGameEntry::new(
                "games.json",
                games.games[0].clone(),
            )]);
            app.dispatch(Action::ToggleWorkspace);
            let out = render(&app, METRICS).unwrap();
            assert_eq!(out.layout.toolbar_targets[6].rect.width, 0);
        }
        app.dispatch(Action::OpenSettings);
        let out = render(
            &app,
            DisplayMetrics {
                width: 2480,
                height: 1860,
                dpi: 300,
            },
        )
        .unwrap();
        if !review {
            assert!(out.layout.settings_chessnut.bottom() < out.layout.settings_close.y);
            let (x, y) = center(out.layout.settings_chessnut);
            assert_eq!(
                out.hit_test_app(x, y, &app),
                Some(HitTarget::ToggleBoardConnection)
            );
            assert_eq!(app.board_status().connection_control_label(), "DISCONNECT");
        }
    }
}
#[test]
fn all_board_session_states_have_reviewed_monochrome_snapshots() {
    let mut actual = Vec::new();
    for small in [false, true] {
        for (name, status) in [
            ("disconnected", Status::Disconnected),
            ("connecting", Status::Connecting),
            ("connected", Status::Connected),
            ("sending", Status::Sending),
            (
                "error",
                Status::Error("Worker lost. Inspect board and reconnect.".into()),
            ),
        ] {
            for settings in [false, true] {
                let mut app = app();
                if small {
                    app.dispatch(Action::ToggleBoardSizeSetting);
                }
                app.dispatch(Action::BoardStatus(Status::Connected));
                app.dispatch(Action::SendBoardPosition);
                app.dispatch(Action::BoardStatus(status.clone()));
                if settings {
                    app.dispatch(Action::OpenSettings);
                }
                let out = render(&app, METRICS).unwrap();
                let name = format!(
                    "{}-{name}-{}",
                    if small { "small" } else { "standard" },
                    if settings { "settings" } else { "board" }
                );
                let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/chessnut-send-forget-snapshots");
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(
                    directory.join(format!("{}.pgm", out.frame.checksum64())),
                    out.frame.to_pgm(),
                )
                .unwrap();
                actual.push((name, out.frame.checksum64()));
            }
        }
    }
    let expected = [
        ("standard-disconnected-board", 9760894365916440813u64),
        ("standard-disconnected-settings", 10641348192551124477u64),
        ("standard-connecting-board", 9760894365916440813u64),
        ("standard-connecting-settings", 13640177509779623532u64),
        ("standard-connected-board", 14107194459146713767u64),
        ("standard-connected-settings", 702989485330028950u64),
        ("standard-sending-board", 4524411084789138271u64),
        ("standard-sending-settings", 15545741498849481846u64),
        ("standard-error-board", 9760894365916440813u64),
        ("standard-error-settings", 1417648665352567393u64),
        ("small-disconnected-board", 12993960575876375483u64),
        ("small-disconnected-settings", 7770809778059619036u64),
        ("small-connecting-board", 12993960575876375483u64),
        ("small-connecting-settings", 16861977923572593641u64),
        ("small-connected-board", 1869262186877601691u64),
        ("small-connected-settings", 9129373288706070905u64),
        ("small-sending-board", 15006747596486280211u64),
        ("small-sending-settings", 9129373288706070905u64),
        ("small-error-board", 12993960575876375483u64),
        ("small-error-settings", 768425456586607496u64),
    ]
    .into_iter()
    .map(|(name, hash)| (name.to_owned(), hash))
    .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn send_and_forget_toolbar_has_no_done_slot() {
    let mut app = app();
    app.dispatch(Action::BoardStatus(Status::Connected));
    let out = render(&app, METRICS).unwrap();
    assert_eq!(out.layout.toolbar_targets.len(), 7);
}

#[test]
fn board_input_states_keep_puzzle_text_visible_in_monochrome_snapshots() {
    use chess_core::{parse_uci_move, BoardInputStatus};
    let mut actual = Vec::new();
    for small in [false, true] {
        for (name, input_status) in [
            ("waiting", BoardInputStatus::WaitingForPosition),
            ("listening", BoardInputStatus::Listening),
            ("correction", BoardInputStatus::MatchKindle),
        ] {
            for settings in [false, true] {
                let mut app = app();
                app.dispatch(Action::ToggleDescription);
                if small {
                    app.dispatch(Action::ToggleBoardSizeSetting);
                }
                app.dispatch(Action::BoardStatus(Status::Connected));
                let baseline = app.board().placement();
                app.dispatch(Action::SendBoardPosition);
                app.dispatch(Action::BoardStatus(Status::Connected));
                if input_status != BoardInputStatus::WaitingForPosition {
                    app.dispatch(Action::BoardPosition(baseline));
                }
                if input_status == BoardInputStatus::MatchKindle {
                    let mut wrong = app.board().clone();
                    wrong.apply_uci_move(parse_uci_move("d7f5").unwrap());
                    app.dispatch(Action::BoardPosition(wrong.placement()));
                }
                assert_eq!(app.board_input_status(), input_status);
                if settings {
                    app.dispatch(Action::OpenSettings);
                }
                let out = render(&app, METRICS).unwrap();
                assert_eq!(out.layout.toolbar_targets.len(), 7);
                let text_panel = out.layout.status;
                if !settings {
                    let (x, y) = center(text_panel);
                    assert_eq!(out.hit_test_app(x, y, &app), None);
                    let (x, y) = center(out.layout.toolbar_targets[6].rect);
                    assert_eq!(
                        out.hit_test_app(x, y, &app),
                        Some(HitTarget::SendBoardPosition)
                    );
                }
                let name = format!(
                    "{}-{name}-{}",
                    if small { "small" } else { "standard" },
                    if settings { "settings" } else { "board" }
                );
                let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/chessnut-board-input-snapshots");
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(
                    directory.join(format!("{}.pgm", out.frame.checksum64())),
                    out.frame.to_pgm(),
                )
                .unwrap();
                actual.push((name, out.frame.checksum64()));
            }
        }
    }
    let expected = [
        ("standard-waiting-board", 2596820488474983898u64),
        ("standard-waiting-settings", 2667761471996937895u64),
        ("standard-listening-board", 2596820488474983898u64),
        ("standard-listening-settings", 2667761471996937895u64),
        ("standard-correction-board", 10308932144499216763u64),
        ("standard-correction-settings", 2667761471996937895u64),
        ("small-waiting-board", 16500098995059119067u64),
        ("small-waiting-settings", 6741614310842997573u64),
        ("small-listening-board", 16500098995059119067u64),
        ("small-listening-settings", 6741614310842997573u64),
        ("small-correction-board", 3441938602462700502u64),
        ("small-correction-settings", 6741614310842997573u64),
    ]
    .into_iter()
    .map(|(name, hash)| (name.to_owned(), hash))
    .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn board_communication_never_replaces_puzzle_text_or_description() {
    for small in [false, true] {
        for notes in [false, true] {
            let mut app = app();
            if small {
                app.dispatch(Action::ToggleBoardSizeSetting);
            }
            if notes {
                app.dispatch(Action::ToggleDescription);
            }
            let expected = render(&app, METRICS).unwrap();
            let status = expected.layout.status;
            for connection in [
                Status::Connected,
                Status::Sending,
                Status::Error("lost worker".into()),
            ] {
                app.dispatch(Action::BoardStatus(Status::Connected));
                app.dispatch(Action::SendBoardPosition);
                app.dispatch(Action::BoardStatus(connection));
                let actual = render(&app, METRICS).unwrap();
                for y in status.y..status.bottom() {
                    for x in status.x..status.right() {
                        assert_eq!(
                            actual.frame.pixel(x, y),
                            expected.frame.pixel(x, y),
                            "board status must not obscure puzzle text"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn board_communication_does_not_mask_analysis_links() {
    let data = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
    let mut app = AppState::new(
        ActiveCollection::from_collection("rich.json", parse_puzzle_file(data).unwrap()),
        Progress::new(),
    );
    app.dispatch(Action::OpenAnalysis);
    for status in [
        Status::Sending,
        Status::Error("lost".into()),
        Status::Connected,
    ] {
        app.dispatch(Action::BoardStatus(Status::Connected));
        app.dispatch(Action::SendBoardPosition);
        app.dispatch(Action::BoardStatus(status));
        let out = render(&app, METRICS).unwrap();
        let chips = &out.analysis.as_ref().unwrap().move_chips;
        assert!(!chips.is_empty());
        for chip in chips {
            let (x, y) = center(chip.rect);
            assert_eq!(
                out.hit_test_app(x, y, &app),
                Some(HitTarget::AnalysisMove(chip.node))
            );
        }
    }
}
