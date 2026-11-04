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
    for c in commands {
        tx.send(Ok((*c).into())).unwrap();
    }
    drop(tx);
    let mut output = Vec::new();
    let result = run_session(
        &mut worker,
        "12:34:56:78:90:ab",
        rx,
        &mut output,
        Arc::new(AtomicBool::new(false)),
        Duration::from_millis(300),
        Duration::ZERO,
    );
    (result, String::from_utf8(output).unwrap())
}
#[test]
fn m3_queries_leds_and_cleanup_use_only_safe_sdk_packets() {
    let (result, out) = run(
        "m3",
        &[
            "telemetry",
            "battery",
            "pieces",
            "mtu",
            "leds e4 green",
            "leds-clear",
            "motion-enable",
            "set-fen bad",
            "stop",
            "quit",
        ],
    );
    assert!(result.is_ok(), "{result:?}: {out}");
    assert!(out.contains("battery percentage=72"));
    assert!(out.contains("pieces count=34"));
    assert!(out.contains("fen_placement=8/8/8/8/8/8/8/8"));
    assert!(out.contains("length=34 kind=WithoutResponse"));
    assert!(out.contains("35_byte_delivery=UNVERIFIED"));
    assert!(out.contains("UNSUPPORTED"));
    assert!(!out.contains("12:34:56:78:90:ab"));
}
#[test]
fn telemetry_failures_are_terminal_and_do_not_replay_queries() {
    for (scenario, reason) in [
        ("m3-no-position", "response timeout kind=0"),
        ("m3-malformed", "MalformedNotification"),
        ("m3-battery-timeout", "response timeout kind=1"),
        ("m3-write-failed", "write_callback_failed"),
        ("m3-disconnect", "Disconnected"),
    ] {
        let (result, out) = run(scenario, &["battery", "battery", "quit"]);
        let error = result.unwrap_err();
        assert!(error.contains(reason), "{scenario}: {error}");
        assert!(
            out.matches("write length=3").count() <= 2,
            "{scenario}: {out}"
        );
    }
}
#[test]
fn unknown_or_small_mtu_refuses_leds_without_fake_delivery() {
    for scenario in ["m3-mtu-unknown", "m3-mtu-small"] {
        let (result, out) = run(scenario, &["mtu", "leds e4 green", "quit"]);
        assert!(result.is_ok(), "{result:?}: {out}");
        assert!(out.contains("LED write not sent"));
        assert!(out.contains("35_byte_delivery=UNVERIFIED"));
    }
}
#[test]
fn invalid_watch_and_led_arguments_send_nothing() {
    let (result, out) = run(
        "m3",
        &[
            "watch 0",
            "watch 301",
            "watch bad",
            "leds z9 green",
            "leds e4 purple",
            "quit",
        ],
    );
    assert!(result.is_ok(), "{result:?}: {out}");
    assert!(!out.contains("write length="));
}

#[test]
fn quiet_watch_completes_and_operator_command_ends_it_promptly() {
    for (seconds, delay, marker) in [
        ("watch 1", 1200, "watch complete position_changes=0"),
        ("watch 300", 150, "mtu="),
    ] {
        let mut worker = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
        worker.arg("m3");
        let (tx, rx) = mpsc::channel();
        tx.send(Ok(seconds.into())).unwrap();
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(delay));
            tx.send(Ok("mtu".into())).unwrap();
            tx.send(Ok("quit".into())).unwrap();
        });
        let mut output = Vec::new();
        let start = std::time::Instant::now();
        let result = run_session(
            &mut worker,
            "12:34:56:78:90:ab",
            rx,
            &mut output,
            Arc::new(AtomicBool::new(false)),
            Duration::from_millis(300),
            Duration::ZERO,
        );
        sender.join().unwrap();
        let out = String::from_utf8(output).unwrap();
        assert!(result.is_ok(), "{result:?}: {out}");
        assert!(out.contains(marker), "{out}");
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
