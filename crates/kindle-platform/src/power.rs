//! Read-only powerd notification adapter; never requests or delays suspend.
use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, Stdio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerEvent {
    Sleeping,
    Awake,
    NativeWakeComplete,
}

impl PowerEvent {
    pub fn parse(line: &str) -> Option<Self> {
        line.split_whitespace().find_map(|word| match word {
            "goingToScreenSaver" => Some(Self::Sleeping),
            "outOfScreenSaver" => Some(Self::Awake),
            "exitingScreenSaver" => Some(Self::NativeWakeComplete),
            _ => None,
        })
    }
}

pub struct PowerEvents {
    child: Child,
    stdout: ChildStdout,
    line: Vec<u8>,
}

impl PowerEvents {
    pub fn open() -> io::Result<Self> {
        // Finite lifetime also bounds an orphan after abrupt parent death.
        let mut child = Command::new("lipc-wait-event")
            .args([
                "-m",
                "-s",
                "60",
                "com.lab126.powerd",
                "goingToScreenSaver,outOfScreenSaver,exitingScreenSaver",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let stdout = child.stdout.take().expect("piped stdout");
        Ok(Self {
            child,
            stdout,
            line: Vec::new(),
        })
    }

    pub(crate) fn stdout(&self) -> &ChildStdout {
        &self.stdout
    }

    /// Read one ready byte, avoiding buffered read-ahead across poll calls.
    pub(crate) fn read_ready(&mut self) -> io::Result<Option<PowerEvent>> {
        let mut byte = [0];
        if self.stdout.read(&mut byte)? == 0 {
            let status = self.child.wait()?;
            // Scribe lipc-wait-event returns 255 for its normal timeout.
            if !status.success() && status.code() != Some(255) {
                return Err(io::Error::other(format!("power listener exited: {status}")));
            }
            *self = Self::open()?;
            return Ok(None);
        }
        if byte[0] == b'\n' {
            let event = PowerEvent::parse(&String::from_utf8_lossy(&self.line));
            self.line.clear();
            return Ok(event);
        }
        if self.line.len() < 4096 {
            self.line.push(byte[0]);
        }
        Ok(None)
    }
}

impl Drop for PowerEvents {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Let the supervisor-owned X server finish native wake before chess redraws.
/// Direct launches without a display handoff have no hook.
pub fn complete_native_wake(power: &mut PowerEvents) -> io::Result<bool> {
    let Some(hook) = std::env::var_os("KINDLE_CHESS_WAKE_HOOK") else {
        return Ok(false);
    };
    let run_hook = |mode| -> io::Result<()> {
        let status = Command::new("sh")
            .arg(&hook)
            .arg(mode)
            .arg(std::process::id().to_string())
            .stdin(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "native wake handoff failed: {status}"
            )));
        }
        Ok(())
    };
    run_hook("--wake-display")?;
    // Already subscribed before CONT, so even immediate completion is retained.
    let result = wait_wake_complete(|remaining| {
        let timeout = remaining.as_millis().clamp(1, i32::MAX as u128) as i32;
        match fbink_sys::wait_pipe(power.stdout(), timeout) {
            Ok(true) => power.read_ready(),
            Ok(false) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native wake completion timed out",
            )),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(None),
            Err(error) => Err(error),
        }
    });
    // On failure normal supervisor cleanup will resume both native processes.
    result?;
    run_hook("--finish-wake-display")?;
    Ok(true)
}

fn wait_wake_complete(
    mut next: impl FnMut(std::time::Duration) -> io::Result<Option<PowerEvent>>,
) -> io::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native wake completion timed out",
            ));
        }
        if next(remaining)? == Some(PowerEvent::NativeWakeComplete) {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod wake_tests {
    use super::*;

    #[test]
    fn completion_waits_past_early_wake_and_partial_lines() {
        let mut events = [
            None,
            Some(PowerEvent::Awake),
            None,
            Some(PowerEvent::NativeWakeComplete),
        ]
        .into_iter();
        wait_wake_complete(|remaining| {
            assert!(remaining <= std::time::Duration::from_secs(4));
            Ok(events.next().expect("must stop at native completion"))
        })
        .unwrap();
        assert!(events.next().is_none());
    }

    #[test]
    fn missing_completion_returns_timeout_for_supervisor_recovery() {
        let error =
            wait_wake_complete(|_| Err(io::Error::from(io::ErrorKind::TimedOut))).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }
}
