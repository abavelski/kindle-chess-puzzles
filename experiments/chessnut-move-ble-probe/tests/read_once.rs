#![cfg(feature = "mock-worker")]
use chessnut_ble_probe::session::run_read_once;
use std::{
    process::Command,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
#[test]
fn unattended_read_collects_status_and_watches_without_stdin_or_early_quit() {
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("m3");
    let mut output = Vec::new();
    let start = Instant::now();
    let result = run_read_once(
        &mut worker,
        "12:34:56:78:90:ab",
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_millis(300),
        Duration::ZERO,
        1,
    );
    let text = String::from_utf8(output).unwrap();
    assert!(result.is_ok(), "{result:?}: {text}");
    assert!(start.elapsed() >= Duration::from_secs(1));
    for expected in [
        "discovery=PASS",
        "battery percentage=72",
        "pieces count=34",
        "fen_placement=",
        "watch complete",
        "connection=DISCONNECTED",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    assert!(!text.contains("write length=34"));
    assert!(!text.contains("12:34:56:78:90:ab"));
}
#[test]
fn unattended_read_reports_failure_without_retry_or_writing_leds() {
    for scenario in ["m3-no-position", "m3-write-failed", "m3-disconnect"] {
        let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
        worker.arg(scenario);
        let mut output = Vec::new();
        let result = run_read_once(
            &mut worker,
            "12:34:56:78:90:ab",
            &mut output,
            Arc::new(AtomicBool::new(false)),
            Duration::from_millis(150),
            Duration::ZERO,
            1,
        );
        assert!(result.is_err(), "{scenario}");
        let text = String::from_utf8(output).unwrap();
        if scenario == "m3-no-position" {
            assert!(
                text.contains("battery percentage=72"),
                "battery query must not depend on an initial position: {text}"
            );
            assert!(text.contains("pieces count=34"));
        }
        assert_eq!(text.matches("connection=CONNECTED").count(), 1);
        assert!(!text.contains("write length=34"));
    }
}

#[test]
fn cli_read_once_runs_with_closed_stdin_and_reports_the_worker_error() {
    use std::{fs, process::Stdio};
    let directory = std::env::temp_dir().join(format!("chessnut-read-once-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let controller = directory.join("chessnut-ble-probe");
    fs::copy(env!("CARGO_BIN_EXE_chessnut-ble-probe"), &controller).unwrap();
    fs::copy(
        env!("CARGO_BIN_EXE_mock-worker"),
        directory.join("kindlebt-worker"),
    )
    .unwrap();
    let result = Command::new(controller)
        .args(["read-once", "--address", "12:34:56:78:90:ab"])
        .env("PROBE_MOCK_SCENARIO", "m3-write-failed")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    fs::remove_dir_all(directory).unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8(result.stderr)
        .unwrap()
        .contains("write_callback_failed"));
}
