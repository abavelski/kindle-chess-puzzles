//! Opt-in board service. The UI never waits on a vendor call or child cleanup.
use chess_core::{BoardIntent, BoardSessionStatus as Status};
use chessnut_board_support::{
    discovery::validate_database,
    ipc::{Client, Operation, Report},
    telemetry::placement,
    TargetFen,
};
use chessnut_move::protocol::{BoardEvent, Command};
use std::{
    process::Command as ProcessCommand,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// `/mnt/us` uses fuse.fsp on the verified Scribe and cannot enforce mode 600.
pub const PRIVATE_CONFIG_PATH: &str = "/var/local/kindle-chess/chessnut-move.private.json";

const OPERATION_MS: u64 = 15000;
const SETTLE_MS: u64 = 500;
const REALTIME_RETRY_MS: u64 = 1000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoardUpdate {
    Status(Status),
    Position(String),
}

pub trait Worker {
    fn request(&mut self, operation: Operation) -> Result<Report, String>;
}
impl Worker for Client {
    fn request(&mut self, operation: Operation) -> Result<Report, String> {
        Client::request(self, operation).map_err(|_| {
            "Board operation failed or timed out. Check board power and reconnect.".into()
        })
    }
}

pub struct Session<W> {
    worker: W,
    status: Status,
    deadline: u64,
    listen_after: Option<u64>,
    target: Option<String>,
    candidate: Option<(String, u64, u64)>,
    emitted: Option<String>,
    pending: Option<String>,
    query_at: u64,
}
impl<W: Worker> Session<W> {
    pub fn new(worker: W) -> Self {
        Self {
            worker,
            status: Status::Disconnected,
            deadline: 0,
            listen_after: None,
            target: None,
            candidate: None,
            emitted: None,
            pending: None,
            query_at: 0,
        }
    }
    pub fn worker(&self) -> &W {
        &self.worker
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn fail(&mut self, reason: String) {
        self.status = Status::Error(reason);
        self.clear_listener();
    }
    fn clear_listener(&mut self) {
        self.listen_after = None;
        self.target = None;
        self.candidate = None;
        self.emitted = None;
        self.pending = None;
    }
    pub fn take_position(&mut self) -> Option<String> {
        self.pending.take()
    }
    fn command(&mut self, command: Command) -> Result<(), String> {
        match self.worker.request(Operation::from_command(&command))? {
            Report::Ok => Ok(()),
            _ => Err("Unexpected board reply. Reconnect.".into()),
        }
    }
    pub fn connect(&mut self, address: String, now: u64) -> Result<(), String> {
        self.status = Status::Connecting;
        if self.worker.request(Operation::Connect { address })? != Report::Connected {
            return Err("Connection not owned. Reconnect.".into());
        }
        let Report::Database { records } = self.worker.request(Operation::Discover)? else {
            return Err("Missing GATT database.".into());
        };
        validate_database(&records).map_err(|_| "Unexpected board services.".to_owned())?;
        // Native Subscribe resolves characteristics, requests MTU 500, awaits
        // matched completion, then completes both CCCD writes in that order.
        if self.worker.request(Operation::Subscribe)? != Report::Ok {
            return Err("Subscription failed.".into());
        }
        if !matches!(
            self.worker.request(Operation::Capabilities)?,
            Report::Capabilities {
                mtu: Some(41..=517),
                callback_status: Some(0)
            }
        ) {
            return Err("MTU negotiation failed. Reconnect.".into());
        }
        self.command(Command::enable_realtime_updates())?;
        self.deadline = now + OPERATION_MS;
        Ok(())
    }
    pub fn observe(&mut self, value: String, length: usize, now: u64) {
        if self.status == Status::Connecting && length == 38 {
            self.status = Status::Connected;
        }
        let Some(after) = self.listen_after else {
            return;
        };
        if now < after || !self.status.connected() {
            return;
        }
        if self.candidate.as_ref().is_some_and(|(_, _, at)| now < *at) {
            return;
        }
        if length != 38 || TargetFen::parse(&format!("{value} w - - 0 1")).is_err() {
            self.candidate = None;
            self.pending = None;
            return;
        }
        match &mut self.candidate {
            Some((placement, _, last)) if placement == &value => *last = now,
            _ => self.candidate = Some((value, now, now)),
        }
    }
    pub fn tick(&mut self, now: u64) {
        if self.status == Status::Connecting && now >= self.deadline {
            self.fail("Board position timed out. Check the board and reconnect.".into());
        }
        if let Some((value, since, _)) = &self.candidate {
            if now.saturating_sub(*since) >= SETTLE_MS && self.emitted.as_ref() != Some(value) {
                self.pending = Some(value.clone());
                self.emitted = Some(value.clone());
                if self.target.as_ref() == Some(value) {
                    self.target = None;
                }
            }
        }
        // Query only: no STOP, motor retry or inference of motion completion.
        if self.target.is_some() && now >= self.query_at && self.status.idle() {
            self.query_at = now + REALTIME_RETRY_MS;
            if let Err(reason) = self.command(Command::enable_realtime_updates()) {
                self.fail(reason);
            }
        }
    }
    pub fn intent(&mut self, intent: BoardIntent, now: u64) -> Result<(), String> {
        match intent {
            BoardIntent::Send(target) if self.status.idle() => {
                let packet = TargetFen::parse(&format!("{target} w - - 0 1"))
                    .map_err(str::to_owned)?
                    .packet();
                self.clear_listener();
                self.status = Status::Sending;
                let started = Instant::now();
                // Each authorized target consumes one Arm/Normal permission. A
                // transport acknowledgement never claims physical completion.
                let result: Result<(), String> = (|| {
                    if self.worker.request(Operation::Arm)? != Report::Ok {
                        return Err("Board arm failed.".into());
                    }
                    self.command(packet)?;
                    self.listen_after = Some(now + started.elapsed().as_millis() as u64 + 1);
                    self.target = Some(target);
                    self.command(Command::enable_realtime_updates())?;
                    self.query_at = now + started.elapsed().as_millis() as u64 + REALTIME_RETRY_MS;
                    Ok(())
                })();
                match result {
                    Ok(()) => self.status = Status::Connected,
                    Err(reason) => {
                        self.fail(reason.clone());
                        return Err(reason);
                    }
                }
            }
            BoardIntent::Disconnect => {
                self.status = Status::Disconnected;
                self.clear_listener();
                self.worker.request(Operation::Disconnect)?;
                self.worker.request(Operation::Shutdown)?;
            }
            _ => return Err("Board is busy or disconnected.".into()),
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateConfig {
    address: String,
}
fn private_address(path: &std::path::Path) -> Result<String, String> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path).map_err(|_| {
        "Missing private board address. Provision chessnut-move.private.json (mode 600).".to_owned()
    })?;
    if !metadata.is_file() || metadata.len() > 1024 || metadata.mode() & 0o077 != 0 {
        return Err(
            "Private board config must be a regular file, mode 600, at most 1024 bytes.".into(),
        );
    }
    let bytes = std::fs::read(path).map_err(|_| "Cannot read private board config.".to_owned())?;
    let config: PrivateConfig = serde_json::from_slice(&bytes)
        .map_err(|_| "Private board config needs one address field.".to_owned())?;
    if config.address.len() != 17
        || !config
            .address
            .split(':')
            .all(|v| v.len() == 2 && v.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("Invalid private board address. Reprovision config.".into());
    }
    Ok(config.address)
}

/// One owner thread and bounded command/status queues. No connection at startup.
pub struct BoardService {
    commands: Option<SyncSender<(u64, BoardIntent)>>,
    updates: Receiver<BoardUpdate>,
    cancelled: Arc<AtomicBool>,
    owner: Option<JoinHandle<()>>,
    generation: Arc<AtomicU64>,
    shutting_down: Arc<AtomicBool>,
}
impl Default for BoardService {
    fn default() -> Self {
        Self::new()
    }
}
impl BoardService {
    pub fn new() -> Self {
        let runtime = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
            .unwrap_or_default();
        Self::with_paths(runtime, PRIVATE_CONFIG_PATH.into())
    }
    /// Platform-owned paths are injectable for subprocess/packaging contracts.
    pub fn with_paths(runtime: std::path::PathBuf, config: std::path::PathBuf) -> Self {
        let (commands, input) = mpsc::sync_channel(16);
        let (output, updates) = mpsc::sync_channel(16);
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancellation = cancelled.clone();
        let generation = Arc::new(AtomicU64::new(0));
        let owner_generation = generation.clone();
        let shutting_down = Arc::new(AtomicBool::new(false));
        let owner_shutdown = shutting_down.clone();
        let owner = thread::spawn(move || {
            service_loop(
                input,
                output,
                cancellation,
                runtime,
                config,
                owner_generation,
                owner_shutdown,
            )
        });
        Self {
            commands: Some(commands),
            updates,
            cancelled,
            owner: Some(owner),
            generation,
            shutting_down,
        }
    }
    pub fn send(&self, intent: BoardIntent) -> Result<(), String> {
        let generation = if intent == BoardIntent::Disconnect {
            self.cancelled.store(true, Ordering::SeqCst);
            self.generation.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.generation.load(Ordering::SeqCst)
        };
        self.commands
            .as_ref()
            .ok_or("Board service closed")?
            .try_send((generation, intent))
            .map_err(|_| "Board service unavailable. Exit and relaunch.".into())
    }
    pub fn poll(&self) -> Result<Option<BoardUpdate>, String> {
        match self.updates.try_recv() {
            Ok(status) => Ok(Some(status)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(_) => Err("Board service exited. Exit and relaunch.".into()),
        }
    }
}
impl Drop for BoardService {
    fn drop(&mut self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        self.cancelled.store(true, Ordering::SeqCst);
        self.commands.take();
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
    }
}
fn drain(active: &mut Session<Client>, start: Instant) {
    for _ in 0..16 {
        match active.worker.next_observed(Duration::from_millis(1)) {
            Ok(Some(observed)) => {
                let now = observed
                    .received_at
                    .saturating_duration_since(start)
                    .as_millis() as u64;
                if let Ok(BoardEvent::PositionChanged(position)) = observed.event {
                    active.observe(placement(&position), observed.bytes.len(), now);
                } else if observed.source == chessnut_board_support::ipc::POSITION_UUID {
                    active.observe(String::new(), 0, now);
                }
            }
            Ok(None) => return,
            Err(_) => {
                active
                    .fail("Board disconnected or worker lost. Inspect board and reconnect.".into());
                return;
            }
        }
    }
    // Do not authorize against a backlog whose timestamps cannot be established.
    active.fail("Telemetry backlog. Reconnect to observe a fresh source.".into());
}
fn service_loop(
    input: Receiver<(u64, BoardIntent)>,
    output: SyncSender<BoardUpdate>,
    cancelled: Arc<AtomicBool>,
    runtime: std::path::PathBuf,
    config: std::path::PathBuf,
    generation: Arc<AtomicU64>,
    shutting_down: Arc<AtomicBool>,
) {
    let start = Instant::now();
    let mut session: Option<Session<Client>> = None;
    let mut published = Status::Disconnected;
    loop {
        if shutting_down.load(Ordering::SeqCst) {
            return;
        }
        let now = start.elapsed().as_millis() as u64;
        let received = input.recv_timeout(Duration::from_millis(20));
        let received = match received {
            Ok((epoch, _)) if epoch != generation.load(Ordering::SeqCst) => continue,
            Ok((epoch, BoardIntent::Connect)) => {
                cancelled.store(false, Ordering::SeqCst);
                if epoch != generation.load(Ordering::SeqCst)
                    || shutting_down.load(Ordering::SeqCst)
                {
                    cancelled.store(true, Ordering::SeqCst);
                    continue;
                }
                Ok(BoardIntent::Connect)
            }
            other => other.map(|(_, intent)| intent),
        };
        match received {
            Ok(BoardIntent::Connect) if session.is_none() => {
                published = Status::Connecting;
                if output
                    .try_send(BoardUpdate::Status(published.clone()))
                    .is_err()
                {
                    return;
                }
                let result = (|| {
                    let address = private_address(&config)?;
                    let mut command = ProcessCommand::new(runtime.join("kindlebt-worker"));
                    command.env("LD_LIBRARY_PATH", runtime.join("lib"));
                    command
                        .arg("--ipc")
                        .arg(runtime.join("lib/libkindlebt.so"))
                        .arg(runtime.join("chessnut-required-libs.txt"));
                    let mut client = Client::spawn_private(&mut command, Duration::from_secs(15))
                        .map_err(|_| {
                        "Cannot launch board worker. Reinstall package.".to_owned()
                    })?;
                    client.supervise_session(cancelled.clone(), Duration::from_secs(55));
                    let mut fresh = Session::new(client);
                    fresh.connect(address, now)?;
                    fresh.deadline = start.elapsed().as_millis() as u64 + OPERATION_MS;
                    Ok::<_, String>(fresh)
                })();
                match result {
                    Ok(fresh) => session = Some(fresh),
                    Err(reason) => {
                        published = Status::Error(reason);
                        if output
                            .try_send(BoardUpdate::Status(published.clone()))
                            .is_err()
                        {
                            return;
                        }
                    }
                }
            }
            Ok(BoardIntent::Disconnect) => {
                published = Status::Disconnected;
                if output
                    .try_send(BoardUpdate::Status(published.clone()))
                    .is_err()
                {
                    return;
                }
                // EOF/TERM invokes the proven native cleanup on its owner thread.
                // Cancellation also interrupts an in-progress connection/request.
                session = None;
            }
            Ok(intent) => {
                if let Some(active) = session.as_mut() {
                    // Reconcile received positions before handling the next explicit tap.
                    drain(active, start);
                    if matches!(intent, BoardIntent::Send(_)) {
                        published = Status::Sending;
                    }
                    if let Err(reason) = active.intent(intent, start.elapsed().as_millis() as u64) {
                        active.fail(reason);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(active) = session.as_mut() {
            drain(active, start);
            active.tick(start.elapsed().as_millis() as u64);
            if active.status != published {
                published = active.status.clone();
                if output
                    .try_send(BoardUpdate::Status(published.clone()))
                    .is_err()
                {
                    return;
                }
            }
            if let Some(position) = active.take_position() {
                if output.try_send(BoardUpdate::Position(position)).is_err() {
                    return;
                }
            }
            if matches!(active.status, Status::Error(_) | Status::Disconnected) {
                session = None;
            }
        }
    }
}
