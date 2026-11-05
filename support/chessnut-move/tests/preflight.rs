use chessnut_board_support::preflight::run_worker;
use std::{process::Command, sync::atomic::AtomicBool, time::Duration};

#[test]
fn preflight_propagates_worker_failure_and_accepts_success() {
    let cancelled = AtomicBool::new(false);
    assert!(run_worker(
        &mut Command::new("true"),
        Duration::from_secs(1),
        &cancelled
    )
    .is_ok());
    assert!(run_worker(
        &mut Command::new("false"),
        Duration::from_secs(1),
        &cancelled
    )
    .unwrap_err()
    .contains("failed"));
}
#[test]
fn preflight_kills_and_reaps_worker_on_timeout_or_interruption() {
    for (flag, timeout, expected) in [
        (false, Duration::from_millis(50), "timed out"),
        (true, Duration::from_secs(2), "interrupted"),
    ] {
        let mut command = Command::new("sleep");
        command.arg("10");
        assert!(run_worker(&mut command, timeout, &AtomicBool::new(flag))
            .unwrap_err()
            .contains(expected));
    }
}
