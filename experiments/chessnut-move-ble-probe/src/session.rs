//! M2 discovery-only controller. No path from shell input to a board write.
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
    client.request(Operation::Connect { address: address.into() }).map_err(|e|format!("connect failed: {e:?}; check current private address, BLE mode and official-app disconnection; no retry performed"))?;
    writeln!(output, "connection=CONNECTED address=[REDACTED]").map_err(|e| e.to_string())?;
    services(&mut client, output)?;
    Ok(client)
}
fn disconnect(client: &mut Option<Client>, output: &mut impl Write) -> Result<(), String> {
    if let Some(mut current) = client.take() {
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
    writeln!(output,"timestamp_unix={timestamp} phase=M2 build={} protocol=1.0.2 address=[REDACTED] writes=DISABLED",env!("CARGO_PKG_VERSION")).map_err(|e|e.to_string())?;
    let mut client = Some(connect(
        worker, address, output, &cancelled, timeout, grace,
    )?);
    writeln!(output,"Commands: services, disconnect, connect (explicit reconnect), quit. Board writes are UNSUPPORTED.").map_err(|e|e.to_string())?;
    let mut prompt = true;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("session interrupted; worker cleanup requested; no writes sent".into());
        }
        if let Some(current) = &mut client {
            current
                .poll_session()
                .map_err(|e| format!("session lost: {e:?}; no automatic reconnect"))?;
        }
        if prompt {
            write!(output, "probe> ")
                .and_then(|()| output.flush())
                .map_err(|e| e.to_string())?;
            prompt = false;
        }
        let line = match input.recv_timeout(Duration::from_millis(25)) {
            Ok(line) => line?,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                disconnect(&mut client, output)?;
                return Ok(());
            }
        };
        prompt = true;
        match line.trim() {
            "quit" | "exit" => {
                disconnect(&mut client, output)?;
                return Ok(());
            }
            "disconnect" => disconnect(&mut client, output)?,
            "connect" if client.is_none() => {
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
                "services | disconnect | connect | quit; writes UNSUPPORTED in M2"
            )
            .map_err(|e| e.to_string())?,
            _ => writeln!(
                output,
                "UNSUPPORTED: M2 permits discovery only; no board writes sent"
            )
            .map_err(|e| e.to_string())?,
        }
    }
}
