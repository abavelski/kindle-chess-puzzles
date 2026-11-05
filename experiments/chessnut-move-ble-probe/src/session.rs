//! Explicit telemetry and human-confirmed M4 controller.
use crate::{
    discovery::validate_database,
    ipc::{Client, Operation, Report},
};
use std::{
    io::Write,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn valid_address(address: &str) -> bool {
    address.len() == 17
        && address.split(':').count() == 6
        && address
            .split(':')
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn services(client: &mut Client, output: &mut impl Write) -> Result<(), String> {
    let Report::Database { records } = client
        .request(Operation::Discover)
        .map_err(|e| format!("discovery failed: {e:?}"))?
    else {
        return Err("native database report required".into());
    };
    for r in &records {
        writeln!(output,"service={} characteristic={} handle={} properties=0x{:02x} WithResponse={} WithoutResponse={} Notify={}",r.service.escape_debug(),r.uuid.escape_debug(),r.handle,r.properties,r.properties & 8 != 0,r.properties & 4 != 0,r.properties & 16 != 0).map_err(|e|e.to_string())?;
    }
    let order = validate_database(&records)
        .map_err(|e| format!("discovery=FAIL reason={e}; stop at H2"))?;
    writeln!(output, "discovery=PASS uuid_byte_order={order} writes=0").map_err(|e| e.to_string())
}
fn connect(
    worker: &mut Command,
    address: &str,
    output: &mut impl Write,
    cancelled: &Arc<AtomicBool>,
    timeout: Duration,
    grace: Duration,
) -> Result<Client, String> {
    let mut client = Client::spawn(worker, timeout)
        .map_err(|e| format!("session worker launch failed: {e:?}"))?;
    client.supervise_session(Arc::clone(cancelled), grace);
    client.request(Operation::Connect { address: address.into() }).map_err(|e|format!("connect failed: {e:?}; connection not confirmed; inspect native connection diagnostics; no retry performed"))?;
    writeln!(output, "connection=CONNECTED address=[REDACTED]").map_err(|e| e.to_string())?;
    services(&mut client, output)?;
    Ok(client)
}
fn disconnect(client: &mut Option<Client>, output: &mut impl Write) -> Result<(), String> {
    if let Some(mut current) = client.take() {
        if current.is_closed() {
            writeln!(
                output,
                "connection=UNVERIFIED worker=CLOSED; see native cleanup diagnostics"
            )
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
        current
            .interrupt()
            .map_err(|e| format!("disconnect/cleanup failed: {e:?}"))?;
        writeln!(output, "connection=DISCONNECTED session=CLOSED").map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn run_session(
    worker: &mut Command,
    address: &str,
    input: Receiver<Result<String, String>>,
    output: &mut impl Write,
    cancelled: Arc<AtomicBool>,
    timeout: Duration,
    grace: Duration,
) -> Result<(), String> {
    if !valid_address(address) {
        return Err(
            "invalid Bluetooth address; expected six hexadecimal octets (address not logged)"
                .into(),
        );
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    writeln!(output,"timestamp_unix={timestamp} phase=M4 build={} protocol=1.0.2 address=[REDACTED] motor_packets=GUARDED_DEFAULT_DISABLED",env!("CARGO_PKG_VERSION")).map_err(|e|e.to_string())?;
    let mut client = Some(connect(
        worker, address, output, &cancelled, timeout, grace,
    )?);
    writeln!(output,"Commands: services, telemetry, battery, pieces, watch 20, mtu, leds e4 green, leds-clear, disconnect, connect, quit. M4: motion-enable, packet-fen <FEN>, set-fen <FEN>, confirm, cancel, stop. Default motion disabled.").map_err(|e|e.to_string())?;
    let mut prompt = true;
    let mut streaming = false;
    let mut motion = crate::motion::Guard::default();
    let mut deferred = None;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(
                "session interrupted; worker cleanup requested; no automatic retries".into(),
            );
        }
        if let Some(current) = &mut client {
            crate::m3_session::drain_background(current, output)?;
        }
        if prompt {
            write!(output, "probe> ")
                .and_then(|()| output.flush())
                .map_err(|e| e.to_string())?;
            prompt = false;
        }
        let line = if let Some(line) = deferred.take() {
            line
        } else {
            match input.recv_timeout(Duration::from_millis(25)) {
                Ok(line) => line?,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    disconnect(&mut client, output)?;
                    return Ok(());
                }
            }
        };
        prompt = true;
        if let Some(current) = &mut client {
            let (handled, next) = crate::motion_session::execute(
                line.trim(),
                &mut motion,
                current,
                &mut streaming,
                &input,
                output,
                timeout,
            )?;
            if handled {
                deferred = next;
                continue;
            }
            let (handled, next) = crate::m3_session::execute(
                line.trim(),
                current,
                &mut streaming,
                &input,
                output,
                timeout,
            )?;
            if handled {
                deferred = next;
                continue;
            }
        }
        match line.trim() {
            "quit" | "exit" => {
                disconnect(&mut client, output)?;
                return Ok(());
            }
            "disconnect" => {motion.reset();disconnect(&mut client, output)?;streaming=false;},
            "connect" if client.is_none() => {
                motion.reset();
                streaming=false;
                client = Some(connect(
                    worker, address, output, &cancelled, timeout, grace,
                )?)
            }
            "connect" => writeln!(output, "Already connected; disconnect first.")
                .map_err(|e| e.to_string())?,
            "services" => {
                if let Some(current) = &mut client {
                    services(current, output)?;
                } else {
                    writeln!(output, "Disconnected; use connect explicitly.")
                        .map_err(|e| e.to_string())?;
                }
            }
            "" | "help" => writeln!(
                output,
                "services | telemetry | battery | pieces | watch 20 | mtu | leds <square> <color> | leds-clear | disconnect | connect | quit; motion-enable | packet-fen <FEN> | set-fen <FEN> | confirm | cancel | stop"
            )
            .map_err(|e| e.to_string())?,
            _ => writeln!(
                output,
                "UNSUPPORTED: no command sent; use help for supported commands"
            )
            .map_err(|e| e.to_string())?,
        }
    }
}

/// One operator-started read test. No stdin thread, retries, LED or motor commands.
pub fn run_read_once(
    worker: &mut Command,
    address: &str,
    output: &mut impl Write,
    cancelled: Arc<AtomicBool>,
    timeout: Duration,
    grace: Duration,
    watch_seconds: u16,
) -> Result<(), String> {
    run_staged_once(
        worker,
        address,
        output,
        cancelled,
        timeout,
        grace,
        (ReadMode::Full, watch_seconds),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadMode {
    Subscriptions,
    Position,
    Full,
}

/// Separate operator-selected tests of the native subscription boundary,
/// position streaming, and the complete telemetry sequence.
pub fn run_staged_once(
    worker: &mut Command,
    address: &str,
    output: &mut impl Write,
    cancelled: Arc<AtomicBool>,
    timeout: Duration,
    grace: Duration,
    plan: (ReadMode, u16),
) -> Result<(), String> {
    let (mode, watch_seconds) = plan;
    if !valid_address(address) || !(1..=300).contains(&watch_seconds) {
        return Err("invalid address or observation duration (1..300 seconds)".into());
    }
    writeln!(output,"READ_ONCE: mode={mode:?}; one connection; no LEDs or motors; wait for read_stage=watch before moving a piece.").map_err(|e|e.to_string())?;
    let mut client = Some(connect(
        worker, address, output, &cancelled, timeout, grace,
    )?);
    let read = crate::m3_session::read_once(
        client.as_mut().unwrap(),
        output,
        timeout,
        watch_seconds,
        mode,
    );
    let cleanup = disconnect(&mut client, output);
    if let Err(reason) = read {
        return Err(match cleanup {
            Ok(()) => reason,
            Err(cleanup) => format!("{reason}; {cleanup}"),
        });
    }
    cleanup
}

/// One user-started guided move with a fixed source and explicit confirmation.
pub fn run_guided_move(
    worker: &mut Command,
    address: &str,
    input: Receiver<Result<String, String>>,
    output: &mut impl Write,
    cancelled: Arc<AtomicBool>,
    settings: (bool, Duration, Duration),
) -> Result<(), String> {
    let (returning, timeout, grace) = settings;
    if !valid_address(address) {
        return Err("invalid Bluetooth address".into());
    }
    let mut client = Some(connect(
        worker, address, output, &cancelled, timeout, grace,
    )?);
    let result = crate::motion_session::guided(
        client.as_mut().unwrap(),
        &input,
        output,
        returning,
        timeout,
        &cancelled,
    );
    let cleanup = disconnect(&mut client, output);
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) => Err(e),
        (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(c)) => Err(format!("{e}; {c}")),
    }
}
