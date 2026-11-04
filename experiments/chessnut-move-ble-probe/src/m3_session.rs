//! Explicit non-motor commands for H3. Wire status and sensor data stay distinct.
use crate::{
    ipc::{Client, Operation, Report},
    telemetry::{self, Observed},
};
use chessnut_move::protocol::{BoardEvent, Command};
use std::{
    io::Write,
    sync::mpsc::{Receiver, TryRecvError},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn error(e: crate::ipc::Error) -> String {
    format!("M3 transport failure: {e:?}; no automatic retries")
}
fn observed(output: &mut impl Write, event: &Observed) -> Result<(), String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let bytes = event
        .bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(
        output,
        "timestamp_unix={stamp} phase=M3 notification source={} length={} bytes={bytes}",
        event.source,
        event.bytes.len()
    )
    .map_err(|e| e.to_string())?;
    match &event.event {
        BoardEvent::PositionChanged(position) => writeln!(
            output,
            "fen_placement={} metadata=unavailable",
            telemetry::placement(position)
        ),
        BoardEvent::BatteryStatus(status) => writeln!(
            output,
            "battery percentage={} charging={}",
            status.percentage, status.charging
        ),
        BoardEvent::PieceStatus(status) => {
            writeln!(output, "pieces count={}", status.pieces.len()).map_err(|e| e.to_string())?;
            for (index, piece) in status.pieces.iter().enumerate() {
                writeln!(
                    output,
                    "piece index={index} color={:?} kind={:?} x={} y={} battery={}",
                    piece.piece.color,
                    piece.piece.kind,
                    piece.x,
                    piece.y,
                    piece
                        .battery_percentage
                        .map_or_else(|| "unavailable".into(), |v| v.to_string())
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
    }
    .map_err(|e: std::io::Error| e.to_string())
}
pub fn drain(client: &mut Client, output: &mut impl Write) -> Result<(), String> {
    for _ in 0..16 {
        client.poll_session().map_err(error)?;
        while let Some(event) = client.take_observed() {
            observed(output, &event)?;
        }
    }
    Ok(())
}
fn send(client: &mut Client, command: &Command, output: &mut impl Write) -> Result<(), String> {
    if !telemetry::m3_command_allowed(command) {
        return Err("M3 motor/unknown packet blocked".into());
    }
    client
        .request(Operation::from_command(command))
        .map_err(error)?;
    writeln!(
        output,
        "write length={} kind={:?} native_api_and_callback=PASS board_delivery=NOT_INFERRED",
        command.bytes().len(),
        command.write_kind()
    )
    .map_err(|e| e.to_string())
}
fn wait_for(
    client: &mut Client,
    output: &mut impl Write,
    kind: u8,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("M3 response timeout kind={kind}; missing position/status or unsupported firmware; no retry"));
        }
        if let Some(event) = client.next_observed(remaining).map_err(error)? {
            let matches = matches!(
                (&event.event, kind),
                (BoardEvent::PositionChanged(_), 0)
                    | (BoardEvent::BatteryStatus(_), 1)
                    | (BoardEvent::PieceStatus(_), 2)
            );
            observed(output, &event)?;
            if matches {
                return Ok(());
            }
        }
    }
}
fn enable(
    client: &mut Client,
    streaming: &mut bool,
    output: &mut impl Write,
    timeout: Duration,
) -> Result<(), String> {
    if *streaming {
        return Ok(());
    }
    drain(client, output)?;
    client.request(Operation::Subscribe).map_err(error)?;
    send(client, &Command::enable_realtime_updates(), output)?;
    wait_for(client, output, 0, timeout)?;
    *streaming = true;
    writeln!(output, "telemetry=ENABLED subscriptions=position,response").map_err(|e| e.to_string())
}
fn capabilities(client: &mut Client, output: &mut impl Write) -> Result<Option<u16>, String> {
    let Report::Capabilities {
        mtu,
        callback_status,
    } = client.request(Operation::Capabilities).map_err(error)?
    else {
        return Err("native MTU report required".into());
    };
    let capacity = mtu.map(|v| v - 3);
    writeln!(output,"mtu={} source=pinned_KindleBT_mtu_callback callback_status={:?} without_response_capacity={} 35_byte_capacity={} 35_byte_delivery=UNVERIFIED motor_packets=BLOCKED",mtu.map_or_else(||"unknown".into(),|v|v.to_string()),callback_status,capacity.map_or_else(||"unknown".into(),|v|v.to_string()),capacity.map_or("unknown",|v|if v>=35{"sufficient_by_MTU"}else{"insufficient"})).map_err(|e|e.to_string())?;
    Ok(mtu)
}
/// Returns (handled, deferred operator command). Watch checks input every 50ms;
/// a new command ends observation and is dispatched by the ordinary shell.
pub fn execute(
    line: &str,
    client: &mut Client,
    streaming: &mut bool,
    input: &Receiver<Result<String, String>>,
    output: &mut impl Write,
    timeout: Duration,
) -> Result<(bool, Option<String>), String> {
    let words: Vec<_> = line.split_whitespace().collect();
    let budget = timeout.min(Duration::from_secs(5));
    match words.as_slice() {
        ["telemetry"] => enable(client, streaming, output, budget)?,
        ["battery"] | ["pieces"] => {
            enable(client, streaming, output, budget)?;
            drain(client, output)?;
            let (command, kind) = if words[0] == "battery" {
                (Command::read_battery_level(), 1)
            } else {
                (Command::read_piece_status(), 2)
            };
            send(client, &command, output)?;
            wait_for(client, output, kind, budget)?;
        }
        ["mtu"] => {
            capabilities(client, output)?;
        }
        ["leds-clear"] | ["leds", _, _] => {
            let command = if words[0] == "leds-clear" {
                telemetry::led_command("clear", "off")
            } else {
                telemetry::led_command(words[1], words[2])
            };
            let command = match command {
                Ok(c) => c,
                Err(reason) => {
                    writeln!(output, "Invalid LED request: {reason}; no write sent")
                        .map_err(|e| e.to_string())?;
                    return Ok((true, None));
                }
            };
            let mtu = capabilities(client, output)?;
            if mtu.is_none_or(|v| v < 37) {
                writeln!(
                    output,
                    "UNSUPPORTED: MTU unknown or below 37; LED write not sent"
                )
                .map_err(|e| e.to_string())?;
            } else {
                enable(client, streaming, output, budget)?;
                send(client, &command, output)?;
            }
        }
        ["watch", seconds] => {
            let seconds = match seconds.parse::<u64>() {
                Ok(v) if (1..=300).contains(&v) => v,
                _ => {
                    writeln!(output, "Invalid watch duration; use 1..300 seconds")
                        .map_err(|e| e.to_string())?;
                    return Ok((true, None));
                }
            };
            enable(client, streaming, output, budget)?;
            let deadline = Instant::now() + Duration::from_secs(seconds);
            let mut positions = 0;
            loop {
                match input.try_recv() {
                    Ok(command) => return Ok((true, Some(command?))),
                    Err(TryRecvError::Disconnected) => return Ok((true, Some("quit".into()))),
                    Err(TryRecvError::Empty) => {}
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                if let Some(event) = client
                    .next_observed(remaining.min(Duration::from_millis(50)))
                    .map_err(error)?
                {
                    if matches!(event.event, BoardEvent::PositionChanged(_)) {
                        positions += 1;
                    }
                    observed(output, &event)?;
                }
            }
            writeln!(output,"watch complete position_changes={positions}; compare sensor placement with physical board").map_err(|e|e.to_string())?;
        }
        _ => return Ok((false, None)),
    }
    Ok((true, None))
}
