//! Read-only powerd notification adapter; never requests or delays suspend.
use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, Stdio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerEvent {
    Sleeping,
    Awake,
}

impl PowerEvent {
    pub fn parse(line: &str) -> Option<Self> {
        line.split_whitespace().find_map(|word| match word {
            "goingToScreenSaver" => Some(Self::Sleeping),
            "outOfScreenSaver" => Some(Self::Awake),
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
                "goingToScreenSaver,outOfScreenSaver",
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
