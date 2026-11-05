#![cfg(feature = "mock-worker")]
use chessnut_ble_probe::session::run_session;
use std::{
    process::Command,
    sync::{atomic::AtomicBool, mpsc, Arc},
    time::Duration,
};
fn run(scenario: &str, commands: &[&str]) -> (Result<(), String>, String) {
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg(scenario);
    let (tx, rx) = mpsc::channel();
    for line in commands {
        tx.send(Ok((*line).to_string())).unwrap();
    }
    drop(tx);
    let mut output = Vec::new();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(1),
        Duration::ZERO,
    );
    (result, String::from_utf8(output).unwrap())
}
#[test]
fn discovery_session_disconnect_reconnect_and_unsupported_writes() {
    let (result, text) = run(
        "database",
        &["services", "force", "disconnect", "connect", "quit"],
    );
    assert!(result.is_ok(), "{result:?}: {text}");
    assert_eq!(text.matches("discovery=PASS").count(), 3);
    assert!(text.contains("WithoutResponse=true"));
    assert!(text.contains("UNSUPPORTED"));
    assert!(!text.contains("12:34:56:78:90:ab"));
}
#[test]
fn missing_uuid_timeout_crash_and_stale_address_fail_without_retry() {
    for scenario in [
        "database-wrong",
        "connect-timeout",
        "connect-crash",
        "connect-error",
    ] {
        let (result, text) = run(scenario, &["connect", "quit"]);
        assert!(result.is_err(), "{scenario}");
        assert!(!text.contains("discovery=PASS"));
        assert!(!text.contains("12:34:56:78:90:ab"));
    }
}
#[test]
fn invalid_address_is_rejected_before_worker_launch() {
    let mut command = Command::new("/nonexistent-worker");
    let (_tx, rx) = mpsc::channel();
    let mut output = Vec::new();
    let result = run_session(
        &mut command,
        "bad address",
        rx,
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_millis(100),
        Duration::ZERO,
    );
    assert!(result.unwrap_err().contains("invalid Bluetooth address"));
}
#[test]
fn cancellation_and_eof_cleanly_close_the_session() {
    let (result, _) = run("database", &[]);
    assert!(result.is_ok());
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("connect-timeout");
    let (_tx, rx) = mpsc::channel();
    let mut output = Vec::new();
    let cancelled = Arc::new(AtomicBool::new(false));
    let trigger = Arc::clone(&cancelled);
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        trigger.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let start = std::time::Instant::now();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut output,
        cancelled,
        Duration::from_secs(5),
        Duration::ZERO,
    );
    assert!(result.is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
    thread.join().unwrap();
}

#[test]
fn idle_disconnect_is_reported_without_waiting_for_operator_input() {
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("database-disconnect");
    let (_tx, rx) = mpsc::channel();
    let mut output = Vec::new();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(2),
        Duration::ZERO,
    );
    assert!(result.unwrap_err().contains("Disconnected"));
}
#[test]
fn controller_signals_request_native_cleanup_and_reap_worker() {
    use std::{fs, process::Stdio, time::Instant};
    for signal in ["-INT", "-TERM", "-HUP"] {
        let directory =
            std::env::temp_dir().join(format!("chessnut-session-{}-{signal}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let controller = directory.join("chessnut-ble-probe");
        fs::copy(env!("CARGO_BIN_EXE_chessnut-ble-probe"), &controller).unwrap();
        fs::copy(
            env!("CARGO_BIN_EXE_mock-worker"),
            directory.join("kindlebt-worker"),
        )
        .unwrap();
        let pid = directory.join("pid");
        let ready = directory.join("ready");
        let cleanup = directory.join("cleanup");
        let mut child = Command::new(controller)
            .args(["session", "--address", "12:34:56:78:90:ab"])
            .env("PROBE_MOCK_SCENARIO", "database-signal")
            .env("PROBE_MOCK_PID_FILE", &pid)
            .env("PROBE_MOCK_READY_FILE", &ready)
            .env("PROBE_MOCK_CLEANUP_FILE", &cleanup)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(ready.exists());
        let worker_pid = fs::read_to_string(pid).unwrap();
        assert!(Command::new("kill")
            .args([signal, &child.id().to_string()])
            .status()
            .unwrap()
            .success());
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = Command::new("kill").args(["-KILL", &worker_pid]).status();
                panic!("signal cleanup timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(!status.success());
        assert_eq!(fs::read_to_string(cleanup).unwrap(), "TERM cleanup");
        assert!(!Command::new("kill")
            .args(["-0", &worker_pid])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success());
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn forced_worker_cleanup_cannot_report_success() {
    let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    worker.arg("database-shutdown-hang");
    let (tx, rx) = mpsc::channel();
    tx.send(Ok("quit".into())).unwrap();
    drop(tx);
    let mut output = Vec::new();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(2),
        Duration::from_millis(50),
    );
    assert!(result.unwrap_err().contains("cleanup"));
}

#[test]
fn native_startup_error_keeps_its_stage_in_the_diagnostic() {
    let (result, _) = run("startup-error", &[]);
    assert!(result.unwrap_err().contains("required_state_symbol"));
}
