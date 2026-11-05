//! Bounded process IPC with terminal failures and a native guarded write policy.
use chessnut_move::protocol::{self, BoardEvent, Command, WriteKind};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, Command as ProcessCommand, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub const MAX_FRAME: usize = 4096;
pub const MAX_PAYLOAD: usize = 512;
pub use chessnut_move::transport::gatt::{
    COMMAND_RESPONSE_UUID as RESPONSE_UUID, COMMAND_SERVICE_UUID as COMMAND_SERVICE,
    COMMAND_WRITE_UUID as WRITE_UUID, POSITION_NOTIFICATION_UUID as POSITION_UUID,
    POSITION_SERVICE_UUID as POSITION_SERVICE,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    WithResponse,
    WithoutResponse,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Connect {
        address: String,
    },
    Discover,
    Subscribe,
    Capabilities,
    Arm,
    Write {
        bytes: Vec<u8>,
        write_kind: WriteMode,
    },
    Disconnect,
    Shutdown,
}
impl Operation {
    pub fn from_command(command: &Command) -> Self {
        Self::Write {
            bytes: command.bytes().to_vec(),
            write_kind: match command.write_kind() {
                WriteKind::WithResponse => WriteMode::WithResponse,
                WriteKind::WithoutResponse => WriteMode::WithoutResponse,
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub v: u8,
    pub id: u32,
    pub operation: Operation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Report {
    Ok,
    Connected,
    Disconnected,
    Services {
        uuids: Vec<String>,
    },
    Database {
        records: Vec<crate::discovery::GattRecord>,
    },
    Capabilities {
        mtu: Option<u16>,
        callback_status: Option<i32>,
    },
    Notification {
        source: String,
        bytes: Vec<u8>,
    },
    Error {
        code: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub v: u8,
    pub id: u32,
    pub report: Report,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Protocol,
    WrongUuid,
    MalformedNotification,
    Worker(String),
    Timeout,
    Disconnected,
    WorkerExited,
    Closed,
    Io,
    Interrupted,
}

pub fn read_report(reader: &mut impl BufRead) -> Result<Envelope, Error> {
    let mut frame = Vec::new();
    let size = reader
        .take((MAX_FRAME + 1) as u64)
        .read_until(b'\n', &mut frame)
        .map_err(|_| Error::Io)?;
    if size == 0 {
        return Err(Error::WorkerExited);
    }
    if size > MAX_FRAME || frame.last() != Some(&b'\n') {
        return Err(Error::Protocol);
    }
    let envelope: Envelope = serde_json::from_slice(&frame).map_err(|_| Error::Protocol)?;
    if envelope.v != 1 {
        return Err(Error::Protocol);
    }
    Ok(envelope)
}

pub struct Client {
    child: Child,
    input: Option<ChildStdin>,
    reports: Receiver<Result<Envelope, Error>>,
    reader: Option<JoinHandle<()>>,
    timeout: Duration,
    id: u32,
    closed: bool,
    cancelled: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    grace: Duration,
    notifications: std::collections::VecDeque<crate::telemetry::Observed>,
}
impl Client {
    pub fn spawn(command: &mut ProcessCommand, timeout: Duration) -> Result<Self, Error> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|_| Error::Io)?;
        let input = child.stdin.take();
        let output = child.stdout.take().ok_or(Error::Io)?;
        let (sender, reports) = mpsc::sync_channel(16);
        let reader = thread::spawn(move || {
            let mut output = BufReader::new(output);
            loop {
                let report = read_report(&mut output);
                let terminal = report.is_err();
                if sender.try_send(report).is_err() || terminal {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            reports,
            reader: Some(reader),
            timeout,
            id: 0,
            closed: false,
            cancelled: None,
            grace: Duration::ZERO,
            notifications: Default::default(),
        })
    }
    pub fn worker_id(&self) -> u32 {
        self.child.id()
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn supervise_session(
        &mut self,
        cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
        grace: Duration,
    ) {
        self.cancelled = Some(cancelled);
        self.grace = grace;
    }
    fn receive(&mut self, deadline: Instant) -> Result<Envelope, Error> {
        loop {
            if self
                .cancelled
                .as_ref()
                .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
            {
                return Err(Error::Interrupted);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::Timeout);
            }
            match self
                .reports
                .recv_timeout(remaining.min(Duration::from_millis(25)))
            {
                Ok(result) => return result,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Err(Error::WorkerExited),
            }
        }
    }
    /// Idle poll: invalid traffic or link loss is terminal, including
    /// while the operator is reading the prompt rather than issuing a request.
    pub fn poll_session(&mut self) -> Result<(), Error> {
        match self.reports.try_recv() {
            Err(mpsc::TryRecvError::Empty) => Ok(()),
            Ok(Ok(Envelope {
                id: 0,
                report: Report::Disconnected,
                ..
            })) => Err(Error::Disconnected),
            Ok(Ok(Envelope {
                id: 0,
                report: Report::Error { code },
                ..
            })) => Err(Error::Worker(code)),
            Ok(Ok(Envelope {
                id: 0,
                report: Report::Notification { source, bytes },
                ..
            })) => {
                if self.notifications.len() >= 16 {
                    return Err(Error::Protocol);
                }
                self.notifications
                    .push_back(crate::telemetry::decode_observed(&source, &bytes)?);
                Ok(())
            }
            Ok(Err(error)) => Err(error),
            Err(mpsc::TryRecvError::Disconnected) => Err(Error::WorkerExited),
            _ => Err(Error::Protocol),
        }
    }
    fn close(&mut self) -> bool {
        if self.closed {
            return true;
        }
        self.closed = true;
        self.input.take();
        if !self.grace.is_zero() && matches!(self.child.try_wait(), Ok(None)) {
            // Fixed shell program, numeric PID passed as an argument. The worker
            // catches TERM and unwinds its owned session; never signal the SSH
            // process group or change the controller's identity.
            let _ = ProcessCommand::new("sh")
                .args([
                    "-c",
                    "kill -TERM \"$1\"",
                    "probe-cleanup",
                    &self.child.id().to_string(),
                ])
                .status();
            let deadline = Instant::now() + self.grace;
            while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(20));
            }
        }
        let clean = self.grace.is_zero()
            || matches!(self.child.try_wait(), Ok(Some(status)) if status.success());
        if !clean && !self.grace.is_zero() {
            eprintln!("phase=M3 cleanup=FAIL worker exit was unsuccessful or forced; verify board is disconnected before retrying");
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        self.notifications.clear();
        clean
    }
    /// Exactly one synchronous request is outstanding. Any error closes the
    /// worker; callers must never retry a motor packet on a replacement worker.
    pub fn request(&mut self, operation: Operation) -> Result<Report, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let shutdown = matches!(operation, Operation::Shutdown);
        let result = self.exchange(operation);
        if (result.is_err() || shutdown) && !self.close() && result.is_ok() {
            return Err(Error::Worker("cleanup_failed_or_forced".into()));
        }
        result
    }
    fn exchange(&mut self, operation: Operation) -> Result<Report, Error> {
        if let Operation::Connect { address } = &operation {
            if address.len() != 17
                || !address
                    .split(':')
                    .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err(Error::Protocol);
            }
        }
        if let Operation::Write { bytes, .. } = &operation {
            if bytes.is_empty() || bytes.len() > protocol::MAX_COMMAND_LEN {
                return Err(Error::Protocol);
            }
        }
        self.id = self.id.checked_add(1).ok_or(Error::Protocol)?;
        let request = Request {
            v: 1,
            id: self.id,
            operation: operation.clone(),
        };
        let mut bytes = serde_json::to_vec(&request).map_err(|_| Error::Protocol)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_FRAME {
            return Err(Error::Protocol);
        }
        self.input
            .as_mut()
            .ok_or(Error::Closed)?
            .write_all(&bytes)
            .map_err(|_| Error::WorkerExited)?;
        let deadline = Instant::now() + self.timeout;
        loop {
            let envelope = self.receive(deadline)?;
            match envelope.report {
                Report::Notification { source, bytes } if envelope.id == 0 => {
                    if self.notifications.len() >= 16 {
                        return Err(Error::Protocol);
                    }
                    self.notifications
                        .push_back(crate::telemetry::decode_observed(&source, &bytes)?);
                }
                Report::Disconnected if envelope.id == 0 => return Err(Error::Disconnected),
                Report::Error { code } if envelope.id == 0 => return Err(Error::Worker(code)),
                report if envelope.id == self.id => {
                    match &report {
                        Report::Error { code } => return Err(Error::Worker(code.clone())),
                        Report::Services { uuids } if matches!(operation, Operation::Discover) => {
                            if ![
                                POSITION_SERVICE,
                                POSITION_UUID,
                                COMMAND_SERVICE,
                                WRITE_UUID,
                                RESPONSE_UUID,
                            ]
                            .iter()
                            .all(|uuid| uuids.iter().any(|found| found.eq_ignore_ascii_case(uuid)))
                            {
                                return Err(Error::WrongUuid);
                            }
                        }
                        Report::Database { records }
                            if matches!(operation, Operation::Discover) =>
                        {
                            if records.len() > 20 {
                                return Err(Error::Protocol);
                            }
                        }
                        Report::Capabilities {
                            mtu,
                            callback_status,
                        } if matches!(operation, Operation::Capabilities) => {
                            if mtu.is_some_and(|v| !(23..=517).contains(&v))
                                || (mtu.is_some() && *callback_status != Some(0))
                            {
                                return Err(Error::Protocol);
                            }
                        }
                        Report::Connected if matches!(operation, Operation::Connect { .. }) => (),
                        Report::Disconnected if matches!(operation, Operation::Disconnect) => (),
                        Report::Ok
                            if matches!(
                                operation,
                                Operation::Subscribe
                                    | Operation::Arm
                                    | Operation::Write { .. }
                                    | Operation::Shutdown
                            ) => {}
                        _ => return Err(Error::Protocol),
                    }
                    return Ok(report);
                }
                _ => return Err(Error::Protocol),
            }
        }
    }
    pub fn take_observed(&mut self) -> Option<crate::telemetry::Observed> {
        self.notifications.pop_front()
    }
    pub fn next_observed(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<crate::telemetry::Observed>, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if let Some(event) = self.notifications.pop_front() {
            return Ok(Some(event));
        }
        let result = (|| {
            let envelope = self.receive(Instant::now() + timeout)?;
            match envelope.report {
                Report::Notification { source, bytes } if envelope.id == 0 => {
                    crate::telemetry::decode_observed(&source, &bytes).map(Some)
                }
                Report::Disconnected if envelope.id == 0 => Err(Error::Disconnected),
                Report::Error { code } if envelope.id == 0 => Err(Error::Worker(code)),
                _ => Err(Error::Protocol),
            }
        })();
        match result {
            Err(Error::Timeout) => Ok(None),
            Err(error) => {
                self.close();
                Err(error)
            }
            value => value,
        }
    }
    pub fn next_notification(&mut self, timeout: Duration) -> Result<BoardEvent, Error> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.next_observed(deadline.saturating_duration_since(Instant::now()))? {
                Some(observed) => {
                    if let Ok(event) = observed.event {
                        return Ok(event);
                    }
                }
                None => {
                    self.close();
                    return Err(Error::Timeout);
                }
            }
            if Instant::now() >= deadline {
                self.close();
                return Err(Error::Timeout);
            }
        }
    }
    /// Host-testable interruption path; live signal wiring belongs to the
    /// native session milestone. Never sends an auto-move or replays a write.
    pub fn interrupt(&mut self) -> Result<(), Error> {
        if self.closed {
            return Ok(());
        }
        self.request(Operation::Disconnect)?;
        self.request(Operation::Shutdown)?;
        Ok(())
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.close();
    }
}
