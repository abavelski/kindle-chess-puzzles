use chess_core::{BoardIntent as Intent, BoardSessionStatus as Status};
use chessnut_board_support::ipc::{Operation, Report, WriteMode};
use kindle_app::chessnut::{BoardUpdate, Session, Worker};
#[derive(Default)]
struct Fake {
    requests: Vec<Operation>,
    fail: bool,
    fail_motor: bool,
    low_mtu: bool,
}
impl Worker for Fake {
    fn request(&mut self, op: Operation) -> Result<Report, String> {
        self.requests.push(op.clone());
        if self.fail
            || (self.fail_motor
                && matches!(&op,Operation::Write {bytes,..} if bytes.len()==35 && bytes[34]==1))
        {
            return Err("lost worker; reconnect".into());
        }
        Ok(match op {
            Operation::Connect { .. } => Report::Connected,
            Operation::Discover => Report::Database {
                records: database(),
            },
            Operation::Capabilities => Report::Capabilities {
                mtu: Some(if self.low_mtu { 23 } else { 300 }),
                callback_status: Some(0),
            },
            Operation::Disconnect => Report::Disconnected,
            _ => Report::Ok,
        })
    }
}
fn database() -> Vec<chessnut_board_support::discovery::GattRecord> {
    use chessnut_board_support::ipc::*;
    [
        (POSITION_SERVICE, POSITION_UUID, 16),
        (COMMAND_SERVICE, WRITE_UUID, 12),
        (COMMAND_SERVICE, RESPONSE_UUID, 16),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(i, (service, uuid, properties))| chessnut_board_support::discovery::GattRecord {
            service: service.into(),
            uuid: uuid.into(),
            properties,
            handle: i as u16 + 1,
        },
    )
    .collect()
}
const SOURCE: &str = "8/8/8/8/8/8/P7/8";
const TARGET: &str = "8/8/8/8/8/P7/8/8";
fn ready() -> Session<Fake> {
    let mut s = Session::new(Fake::default());
    s.connect("00:11:22:33:44:55".into(), 0).unwrap();
    assert_eq!(s.status(), &Status::Connecting);
    s.observe(SOURCE.into(), 38, 1);
    assert_eq!(s.status(), &Status::Connected);
    s
}
#[test]
fn asynchronous_service_missing_private_config_and_dropped_worker_are_actionable() {
    use kindle_app::chessnut::BoardService;
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let folder = std::env::temp_dir().join(format!("chessnut-service-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let config = folder.join("config.json");
    let await_error = |service: &BoardService| {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(BoardUpdate::Status(Status::Error(reason))) = service.poll().unwrap() {
                assert!(!reason.is_empty());
                break;
            }
            assert!(
                Instant::now() < deadline,
                "UI update must not require a touch"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    let service = BoardService::with_paths(folder.clone(), config.clone());
    service.send(Intent::Connect).unwrap();
    await_error(&service);
    drop(service);
    std::fs::write(&config, br#"{"address":"00:11:22:33:44:55"}"#).unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let worker = folder.join("kindlebt-worker");
    std::fs::write(&worker, "#!/bin/sh\nexit 1\n").unwrap();
    std::fs::set_permissions(&worker, std::fs::Permissions::from_mode(0o755)).unwrap();
    let service = BoardService::with_paths(folder.clone(), config.clone());
    service.send(Intent::Connect).unwrap();
    await_error(&service);
    drop(service);
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o644)).unwrap();
    let service = BoardService::with_paths(folder.clone(), config);
    service.send(Intent::Connect).unwrap();
    await_error(&service);
    drop(service);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn failed_motor_write_is_never_retried_and_low_mtu_never_becomes_ready() {
    let mut s = Session::new(Fake {
        fail_motor: true,
        ..Fake::default()
    });
    s.connect("00:11:22:33:44:55".into(), 0).unwrap();
    s.observe(SOURCE.into(), 38, 1);
    assert!(s.intent(Intent::Send(TARGET.into()), 2).is_err());
    assert!(s.intent(Intent::Send(TARGET.into()), 4).is_err());
    assert_eq!(
        s.worker()
            .requests
            .iter()
            .filter(|op| matches!(op,Operation::Write {bytes,..} if bytes.len()==35))
            .count(),
        1
    );
    let mut s = Session::new(Fake {
        low_mtu: true,
        ..Fake::default()
    });
    assert!(s.connect("00:11:22:33:44:55".into(), 0).is_err());
    assert!(!s.status().idle());
    assert!(!s
        .worker()
        .requests
        .iter()
        .any(|op| matches!(op, Operation::Write { .. })));
}

#[test]
fn production_private_config_uses_permission_enforcing_storage_outside_usb() {
    let path = std::path::Path::new(kindle_app::chessnut::PRIVATE_CONFIG_PATH);
    assert!(path.is_absolute());
    assert!(
        !path.starts_with("/mnt/us"),
        "Scribe fuse.fsp reports mode 777 despite chmod"
    );
    assert_eq!(
        path,
        std::path::Path::new("/var/local/kindle-chess/chessnut-move.private.json")
    );
}

#[test]
fn idle_connection_survives_silent_or_partial_telemetry_until_explicit_disconnect() {
    let mut s = ready();
    s.tick(3_600_000);
    assert_eq!(s.status(), &Status::Connected);
    s.observe(String::new(), 20, 3_600_001);
    s.tick(3_700_000);
    assert_eq!(s.status(), &Status::Connected);
    assert!(!s
        .worker()
        .requests
        .iter()
        .any(|op| matches!(op, Operation::Disconnect | Operation::Shutdown)));
    s.intent(Intent::Disconnect, 3_700_001).unwrap();
    assert_eq!(s.status(), &Status::Disconnected);
}

#[test]
fn send_and_forget_allows_next_explicit_target_without_done_or_new_telemetry() {
    let mut s = ready();
    let initial = s.worker().requests.len();
    s.intent(Intent::Send(TARGET.into()), 4000).unwrap();
    assert_eq!(s.status(), &Status::Connected);
    s.intent(Intent::Send(SOURCE.into()), 3_600_001).unwrap();
    assert_eq!(s.status(), &Status::Connected);
    let requests = &s.worker().requests[initial..];
    assert_eq!(
        requests.len(),
        6,
        "two explicit Arm/Normal/realtime sequences, no STOP"
    );
    for pair in requests.chunks_exact(3) {
        assert_eq!(pair[0], Operation::Arm);
        assert_eq!(
            pair[2],
            Operation::from_command(&chessnut_move::protocol::Command::enable_realtime_updates())
        );
        assert!(
            matches!(&pair[1], Operation::Write { bytes, write_kind: WriteMode::WithoutResponse } if bytes.len()==35 && bytes[34]==1)
        );
    }
}

#[test]
fn initial_readiness_requires_complete_report_and_has_a_finite_timeout() {
    let mut s = Session::new(Fake::default());
    s.connect("00:11:22:33:44:55".into(), 0).unwrap();
    s.observe(String::new(), 20, 1);
    assert_eq!(s.status(), &Status::Connecting);
    assert!(s.intent(Intent::Send(TARGET.into()), 2).is_err());
    s.tick(15000);
    assert!(matches!(s.status(), Status::Error(_)));
}

#[test]
fn silence_or_partial_reports_after_send_never_queue_or_replay_a_target() {
    let mut s = ready();
    s.intent(Intent::Send(TARGET.into()), 2).unwrap();
    let submitted = s
        .worker()
        .requests
        .iter()
        .filter(|r| matches!(r, Operation::Write { bytes, .. } if bytes.len()==35))
        .count();
    s.observe(String::new(), 20, 3);
    s.tick(3_600_000);
    s.observe(SOURCE.into(), 38, 3_600_001);
    assert_eq!(s.status(), &Status::Connected);
    assert_eq!(
        s.worker()
            .requests
            .iter()
            .filter(|r| matches!(r, Operation::Write { bytes, .. } if bytes.len()==35))
            .count(),
        submitted
    );
    s.intent(Intent::Disconnect, 3_600_002).unwrap();
    assert!(s.intent(Intent::Send(TARGET.into()), 3_600_003).is_err());
}

#[test]
fn service_acknowledges_each_submission_even_when_connection_status_is_unchanged() {
    use kindle_app::chessnut::BoardService;
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let folder = std::env::temp_dir().join(format!("chessnut-send-forget-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let config = folder.join("config.json");
    std::fs::write(&config, br#"{"address":"00:11:22:33:44:55"}"#).unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let script = r#"#!/usr/bin/env python3
import json,sys
records=RECORDS
source=SOURCE_UUID
for line in sys.stdin:
    request=json.loads(line)
    op=request['operation'];kind=op['type']
    report={'type':'ok'}
    if kind=='connect': report={'type':'connected'}
    elif kind=='discover': report={'type':'database','records':records}
    elif kind=='capabilities': report={'type':'capabilities','mtu':300,'callback_status':0}
    print(json.dumps({'v':1,'id':request['id'],'report':report}),flush=True)
    if kind=='write' and op['bytes']==[33,1,0]:
        print(json.dumps({'v':1,'id':0,'report':{'type':'notification','source':source,'bytes':[1,36]+[0]*36}}),flush=True)
"#.replace("RECORDS", &serde_json::to_string(&database()).unwrap())
    .replace("SOURCE_UUID", &serde_json::to_string(chessnut_board_support::ipc::POSITION_UUID).unwrap());
    let worker = folder.join("kindlebt-worker");
    std::fs::write(&worker, script).unwrap();
    std::fs::set_permissions(&worker, std::fs::Permissions::from_mode(0o755)).unwrap();
    let service = BoardService::with_paths(folder.clone(), config);
    let wait_connected = || {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(BoardUpdate::Status(status)) = service.poll().unwrap() {
                assert!(!matches!(status, Status::Error(_)), "{status:?}");
                if status == Status::Connected {
                    break;
                }
            }
            assert!(
                Instant::now() < deadline,
                "every send must acknowledge completion to unlock the UI"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    service.send(Intent::Connect).unwrap();
    wait_connected();
    for target in [TARGET, SOURCE] {
        service.send(Intent::Send(target.into())).unwrap();
        wait_connected();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(BoardUpdate::Position(position)) = service.poll().unwrap() {
                assert_eq!(position, "8/8/8/8/8/8/8/8");
                break;
            }
            assert!(
                Instant::now() < deadline,
                "subprocess reports must reach the UI without touch input"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    drop(service);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn after_send_only_new_complete_settled_reports_are_forwarded_and_only_realtime_is_retried() {
    let mut s = ready();
    s.observe(SOURCE.into(), 38, 2);
    s.intent(Intent::Send(TARGET.into()), 10).unwrap();
    s.observe(SOURCE.into(), 38, 2); // buffered before submit
    s.tick(600);
    assert_eq!(s.take_position(), None);
    s.observe(TARGET.into(), 38, 700);
    s.observe(TARGET.into(), 38, 900); // identical report does not restart quiet window
    s.tick(1199);
    assert_eq!(s.take_position(), None);
    s.tick(1200);
    assert_eq!(s.take_position(), Some(TARGET.into()));
    s.tick(3000);
    assert_eq!(s.take_position(), None);
    s.observe(SOURCE.into(), 38, 3100);
    s.observe(String::new(), 20, 3200);
    s.tick(3700);
    assert_eq!(s.take_position(), None);
    s.observe(SOURCE.into(), 38, 3800);
    s.tick(4300);
    assert_eq!(s.take_position(), Some(SOURCE.into()));
    s.intent(Intent::Send(SOURCE.into()), 4400).unwrap();
    s.tick(6500); // only realtime retry; no Normal replay or STOP
    let writes: Vec<_> = s
        .worker()
        .requests
        .iter()
        .filter_map(|r| {
            if let Operation::Write { bytes, .. } = r {
                Some(bytes)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(writes.iter().filter(|b| b.len() == 35).count(), 2);
    assert!(writes
        .iter()
        .all(|b| b.len() == 35 && b[34] == 1 || b.as_slice() == [0x21, 1, 0]));
    assert_eq!(s.status(), &Status::Connected);
}

#[test]
fn physical_solver_reply_and_next_move_round_trip_without_manual_sync_or_echo_grading() {
    use chess_core::{
        parse_puzzle_file, parse_uci_move, Action, ActiveCollection, AppState, BoardInputStatus,
        Effect, Progress,
    };
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let mut session = ready();
    app.dispatch(Action::NextPuzzle);
    app.dispatch(Action::BoardStatus(Status::Connected));
    for effect in app.dispatch(Action::SendBoardPosition) {
        if let Effect::Board(intent) = effect {
            session.intent(intent, 10).unwrap();
        }
    }
    app.dispatch(Action::BoardStatus(session.status().clone()));
    session.observe(app.board().placement(), 38, 1000);
    session.tick(1500);
    app.dispatch(Action::BoardPosition(session.take_position().unwrap()));
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    let mut physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e2e6").unwrap());
    session.observe(physical.placement(), 38, 2000);
    session.tick(2500);
    let effects = app.dispatch(Action::BoardPosition(session.take_position().unwrap()));
    assert!(matches!(
        effects.as_slice(),
        [Effect::Board(Intent::Send(_))]
    ));
    for effect in effects {
        if let Effect::Board(intent) = effect {
            session.intent(intent, 2600).unwrap();
        }
    }
    app.dispatch(Action::BoardStatus(session.status().clone()));
    session.observe(physical.placement(), 38, 2700);
    session.tick(3200);
    assert!(app
        .dispatch(Action::BoardPosition(session.take_position().unwrap()))
        .is_empty());
    assert_eq!(app.solution_ply(), 2);
    session.observe(app.board().placement(), 38, 3300);
    session.tick(3800);
    assert!(app
        .dispatch(Action::BoardPosition(session.take_position().unwrap()))
        .is_empty());
    assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
    physical = app.board().clone();
    physical.apply_uci_move(parse_uci_move("e6f7").unwrap());
    session.observe(physical.placement(), 38, 4000);
    session.tick(4500);
    assert_eq!(
        app.dispatch(Action::BoardPosition(session.take_position().unwrap())),
        vec![Effect::ProgressChanged]
    );
    assert_eq!(app.solution_ply(), 3);
    assert_eq!(
        session
            .worker()
            .requests
            .iter()
            .filter(|r| matches!(r,Operation::Write{bytes,..} if bytes.len()==35))
            .count(),
        2
    );
}

#[test]
fn seating_rotation_round_trips_real_sdk_packets_sensor_reports_and_reply_targets() {
    use chess_core::{
        parse_puzzle_file, parse_uci_move, Action, ActiveCollection, AppState, BoardInputStatus,
        Effect, PhysicalBoardSeat as Seat, Progress, Settings,
    };
    use chessnut_board_support::{
        telemetry::{decode_observed, placement},
        TargetFen,
    };
    use chessnut_move::protocol::BoardEvent;
    for seat in [Seat::White, Seat::Black, Seat::Changing] {
        for black in [false, true] {
            let data = format!(
                r#"{{"version":1,"puzzles":[{{"id":"seat","fen":"7k/8/8/8/8/8/P6p/K7 {} - - 0 1","solution":{}}}]}}"#,
                if black { "b" } else { "w" },
                if black {
                    r#"["h2h1q","a2a3","h1h2"]"#
                } else {
                    r#"["a2a3","h2h1q","a3a4"]"#
                }
            );
            let mut settings = Settings::default();
            settings.set_physical_board_seat(seat);
            let mut app = AppState::new_with_settings(
                ActiveCollection::from_collection(
                    "seat.json",
                    parse_puzzle_file(data.as_bytes()).unwrap(),
                ),
                Progress::new(),
                settings,
            );
            let rotated = seat.rotates(app.active_puzzle().side_to_move());
            let mut session = ready();
            app.dispatch(Action::BoardStatus(Status::Connected));
            let submit = |session: &mut Session<Fake>, effects: Vec<Effect>, now| {
                for effect in effects {
                    if let Effect::Board(intent) = effect {
                        session.intent(intent, now).unwrap();
                    }
                }
            };
            let report = |wire: &str| {
                let packet = TargetFen::parse(&format!("{wire} w - - 0 1"))
                    .unwrap()
                    .packet();
                let mut bytes = vec![1, 0x24];
                bytes.extend_from_slice(&packet.bytes()[2..34]);
                bytes.extend([0; 4]);
                let event = decode_observed(chessnut_board_support::ipc::POSITION_UUID, &bytes)
                    .unwrap()
                    .event
                    .unwrap();
                let BoardEvent::PositionChanged(position) = event else {
                    panic!("position report")
                };
                let decoded = placement(&position);
                assert_eq!(decoded, wire);
                decoded
            };
            submit(&mut session, app.dispatch(Action::SendBoardPosition), 10);
            app.dispatch(Action::BoardStatus(session.status().clone()));
            session.observe(report(&app.board().placement_rotated(rotated)), 38, 1000);
            session.tick(1500);
            app.dispatch(Action::BoardPosition(session.take_position().unwrap()));
            assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
            let mut physical = app.board().clone();
            physical.apply_uci_move(parse_uci_move(if black { "h2h1q" } else { "a2a3" }).unwrap());
            session.observe(report(&physical.placement_rotated(rotated)), 38, 2000);
            session.tick(2500);
            let effects = app.dispatch(Action::BoardPosition(session.take_position().unwrap()));
            assert_eq!(app.solution_ply(), 2);
            submit(&mut session, effects, 2600);
            app.dispatch(Action::BoardStatus(session.status().clone()));
            session.observe(report(&app.board().placement_rotated(rotated)), 38, 3300);
            session.tick(3800);
            app.dispatch(Action::BoardPosition(session.take_position().unwrap()));
            assert_eq!(app.board_input_status(), BoardInputStatus::Listening);
            let motor = session
                .worker()
                .requests
                .iter()
                .filter_map(|r| {
                    if let Operation::Write { bytes, .. } = r {
                        if bytes.len() == 35 {
                            return Some(bytes.clone());
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();
            assert_eq!(motor.len(), 2);
            assert_eq!(
                motor[1],
                TargetFen::parse(&format!(
                    "{} w - - 0 1",
                    app.board().placement_rotated(rotated)
                ))
                .unwrap()
                .packet()
                .bytes()
            );
        }
    }
}
