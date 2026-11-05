//! Bounded child supervision for the harmless, separately privileged C preflight.
use std::{
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub fn run_worker(
    command: &mut Command,
    timeout: Duration,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let mut child = command
        .stdin(Stdio::null())
        .spawn()
        .map_err(|error| format!("preflight worker launch failed: {error}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        let reason = if cancelled.load(Ordering::Relaxed) {
            Some("preflight interrupted")
        } else if Instant::now() >= deadline {
            Some("preflight timed out")
        } else {
            None
        };
        if let Some(reason) = reason {
            let _ = child.kill();
            let _ = child.wait();
            return Err(reason.into());
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => return Err(format!("preflight worker failed: {status}")),
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("preflight worker wait failed: {error}"));
            }
        }
    }
}
