#![cfg(feature = "mock-worker")]
use chessnut_ble_probe::session::run_session;
use std::{
    process::Command,
    sync::{atomic::AtomicBool, mpsc, Arc},
    time::Duration,
};
fn run(
    scenario: &str,
    commands: Vec<&'static str>,
    pause: Duration,
) -> (Result<(), String>, String) {
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg(scenario);
    let (tx, rx) = mpsc::channel();
    let producer = std::thread::spawn(move || {
        for c in commands {
            if tx.send(Ok(c.to_owned())).is_err() {
                return;
            }
        }
        std::thread::sleep(pause);
        let _ = tx.send(Ok("quit".into()));
    });
    let mut out = Vec::new();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut out,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(1),
        Duration::ZERO,
    );
    producer.join().unwrap();
    (result, String::from_utf8(out).unwrap())
}
const TARGET: &str = "set-fen 4k3/8/8/8/4P3/8/8/4K3 w - - 0 1";
#[test]
fn preview_does_not_write_motors_and_confirmation_verifies_sensor_target() {
    let (r, out) = run(
        "m4",
        vec!["motion-enable", TARGET],
        Duration::from_millis(100),
    );
    assert!(r.is_ok(), "{r:?} {out}");
    assert!(out.contains("No motor write yet"));
    assert!(!out.contains("phase=M4 write length=35"));
    let (r, out) = run(
        "m4",
        vec!["motion-enable", TARGET, "confirm"],
        Duration::from_millis(250),
    );
    assert!(r.is_ok(), "{r:?} {out}");
    assert!(out.contains("motion=TARGET_OBSERVED"), "{out}");
    assert_eq!(
        out.matches("phase=M4 write length=35").count(),
        2,
        "one motor command and one STOP: {out}"
    );
}
#[test]
fn pending_input_stops_inflight_motion_and_does_not_dispatch_second_target() {
    let (r, out) = run(
        "m4-quiet",
        vec!["motion-enable", TARGET, "confirm", "stop"],
        Duration::from_millis(100),
    );
    assert!(r.is_ok(), "{r:?} {out}");
    assert!(out.contains("motion=STOP_REQUESTED"), "{out}");
    assert!(!out.contains("motion=TARGET_OBSERVED"));
}
#[test]
fn timeout_sends_stop_and_has_no_automatic_replay() {
    let (r, out) = run(
        "m4-quiet",
        vec!["motion-enable", TARGET, "confirm"],
        Duration::from_millis(2200),
    );
    assert!(r.unwrap_err().contains("motion timeout"), "{out}");
    assert_eq!(out.matches("phase=M4 write length=35").count(), 2, "{out}");
}
#[test]
fn cancel_and_no_enable_never_dispatch_a_motor_command() {
    let (r, out) = run(
        "m4",
        vec!["motion-enable", TARGET, "cancel", "confirm"],
        Duration::from_millis(100),
    );
    assert!(r.is_ok(), "{r:?} {out}");
    assert!(out.contains("confirmation=REFUSED"));
    assert!(!out.contains("phase=M4 write length=35"));
    let (r, out) = run("m4", vec![TARGET], Duration::from_millis(100));
    assert!(r.unwrap_err().contains("motion disabled"));
    assert!(!out.contains("phase=M4 write length=35"));
}
#[test]
fn guided_script_requires_confirmation_and_checks_fixed_source() {
    use chessnut_ble_probe::session::run_guided_move;
    for (scenario, approval, success, returning) in [
        ("m4", "cancel", true, false),
        ("m4", "confirm", true, false),
        ("m4-return", "confirm", true, true),
        ("m4-wrong-source", "confirm", false, false),
    ] {
        let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
        worker.arg(scenario);
        let (tx, rx) = mpsc::channel();
        tx.send(Ok(approval.into())).unwrap();
        let keeper = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            drop(tx);
        });
        let mut out = Vec::new();
        let result = run_guided_move(
            &mut worker,
            "12:34:56:78:90:ab",
            rx,
            &mut out,
            Arc::new(AtomicBool::new(false)),
            (returning, Duration::from_secs(1), Duration::ZERO),
        );
        keeper.join().unwrap();
        let out = String::from_utf8(out).unwrap();
        assert_eq!(result.is_ok(), success, "{result:?} {out}");
        if success {
            if returning {
                assert!(out.contains("SOURCE: 4k3/4P3/8/8/8/8/8/4K3"), "{out}");
            } else {
                assert!(
                    out.contains("TARGET: 4k3/4P3/8/8/8/8/8/4K3 w - - 0 1"),
                    "{out}"
                );
            }
        }

        assert_eq!(
            out.contains("motion=TARGET_OBSERVED"),
            success && approval == "confirm",
            "{out}"
        );
        if approval == "cancel" || !success {
            assert!(!out.contains("phase=M4 write length=35"), "{out}");
        }
    }
}
#[test]
fn guided_confirmation_prompt_stays_quiet_while_source_reports_continue() {
    use chessnut_ble_probe::session::run_guided_move;
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("m4-stream");
    let (tx, rx) = mpsc::channel();
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        let _ = tx.send(Ok("cancel".into()));
    });
    let mut out = Vec::new();
    let result = run_guided_move(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut out,
        Arc::new(AtomicBool::new(false)),
        (false, Duration::from_secs(1), Duration::ZERO),
    );
    producer.join().unwrap();
    let out = String::from_utf8(out).unwrap();
    assert!(result.is_ok(), "{result:?} {out}");
    assert!(out.contains("move=CANCELLED"));
    assert!(
        out.matches("fen_placement=").count() <= 1,
        "background stream flooded output: {out}"
    );
}
#[test]
fn paused_motion_reports_are_resumed_and_verified_after_human_done() {
    let (r, out) = run(
        "m4-paused",
        vec!["motion-enable", TARGET, "confirm", "done"],
        Duration::from_millis(250),
    );
    assert!(r.is_ok(), "{r:?} {out}");
    assert!(out.contains("verification=after_human_done"), "{out}");
    assert!(out.contains("motion=TARGET_OBSERVED"), "{out}");
    assert_eq!(
        out.matches("phase=M4 write length=35").count(),
        2,
        "only original motor write and STOP: {out}"
    );
}
#[test]
fn premature_done_cannot_turn_a_wrong_position_into_success() {
    let (r, out) = run(
        "m4-quiet",
        vec!["motion-enable", TARGET, "confirm", "done"],
        Duration::from_millis(200),
    );
    assert!(
        r.unwrap_err().contains("post-STOP target mismatch"),
        "{out}"
    );
    assert!(!out.contains("motion=TARGET_OBSERVED"));
    assert_eq!(out.matches("phase=M4 write length=35").count(), 2);
}
#[test]
fn guided_done_handles_firmware_that_pauses_reports() {
    use chessnut_ble_probe::session::run_guided_move;
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("m4-paused");
    let (tx, rx) = mpsc::channel();
    tx.send(Ok("confirm".into())).unwrap();
    tx.send(Ok("done".into())).unwrap();
    let keeper = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        drop(tx);
    });
    let mut out = Vec::new();
    let result = run_guided_move(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut out,
        Arc::new(AtomicBool::new(false)),
        (false, Duration::from_secs(1), Duration::ZERO),
    );
    keeper.join().unwrap();
    let out = String::from_utf8(out).unwrap();
    assert!(result.is_ok(), "{result:?} {out}");
    assert!(out.contains("verification=after_human_done"), "{out}");
    assert_eq!(out.matches("phase=M4 write length=35").count(), 2);
}
