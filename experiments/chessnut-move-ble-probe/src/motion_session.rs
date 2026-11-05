//! Human-confirmed M4 shell. No command is replayed after a transport failure.
use crate::{
    ipc::{Client, Operation, Report},
    motion::{Guard, Plan},
    telemetry, TargetFen,
};
use chessnut_move::protocol::{BoardEvent, Command, Position};
use std::{
    io::Write,
    sync::mpsc::{Receiver, TryRecvError},
    time::{Duration, Instant},
};
fn send(client: &mut Client, command: &Command, out: &mut impl Write) -> Result<(), String> {
    client
        .request(Operation::from_command(command))
        .map_err(|e| format!("M4 write failed: {e:?}; no retry"))?;
    writeln!(
        out,
        "phase=M4 write length={} kind={:?} api_and_callback=PASS physical_delivery=UNVERIFIED",
        command.bytes().len(),
        command.write_kind()
    )
    .map_err(|e| e.to_string())
}
fn discard_background(client: &mut Client) -> Result<(), String> {
    for _ in 0..16 {
        client
            .poll_session()
            .map_err(|e| format!("M4 background sensor: {e:?}"))?;
        while let Some(event) = client.take_observed() {
            if event.source == crate::ipc::POSITION_UUID && event.event.is_err() {
                return Err("incomplete position in motion workflow; abort, native cleanup requests STOP if needed".into());
            }
        }
    }
    Ok(())
}
fn fresh(client: &mut Client) -> Result<Position, String> {
    // Discard already-buffered positions, then request realtime reports once.
    discard_background(client)?;
    client
        .request(Operation::from_command(&Command::enable_realtime_updates()))
        .map_err(|e| format!("M4 realtime request: {e:?}"))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if Instant::now() >= deadline {
            return Err("M4 fresh position timeout; nothing sent".into());
        }
        if let Some(event) = client
            .next_observed(deadline.saturating_duration_since(Instant::now()))
            .map_err(|e| format!("M4 sensor: {e:?}"))?
        {
            match event.event {
                Ok(BoardEvent::PositionChanged(p)) => return Ok(p),
                Err(_) if event.source == crate::ipc::POSITION_UUID => {
                    return Err("incomplete sensor position; nothing sent".into())
                }
                _ => {}
            }
        }
    }
}
fn verify_after_done(
    client: &mut Client,
    target: Position,
    out: &mut impl Write,
    start: Instant,
) -> Result<(), String> {
    // Human has observed completion. STOP first, then re-enable the known
    // realtime stream; never send another auto-move to obtain feedback.
    let first = fresh(client)?;
    if first != target {
        return Err(format!(
            "post-STOP target mismatch: expected={} observed={}; no retry",
            telemetry::placement(&target),
            telemetry::placement(&first)
        ));
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut matches = 1;
    while matches < 3 {
        if Instant::now() >= deadline {
            return Err("post-STOP position verification timeout; no retry".into());
        }
        if let Some(event) = client
            .next_observed(deadline.saturating_duration_since(Instant::now()))
            .map_err(|e| format!("post-STOP verification: {e:?}"))?
        {
            match event.event {
                Ok(BoardEvent::PositionChanged(p)) if p == target => matches += 1,
                Ok(BoardEvent::PositionChanged(p)) => {
                    return Err(format!(
                        "post-STOP target changed: observed={}; no retry",
                        telemetry::placement(&p)
                    ))
                }
                Err(e) if event.source == crate::ipc::POSITION_UUID => {
                    return Err(format!("post-STOP sensor incomplete: {e}"))
                }
                _ => {}
            }
        }
    }
    writeln!(out,"motion=TARGET_OBSERVED verification=after_human_done consecutive_reports=3 elapsed_until_verification_ms={} placement={} motion_permission=DISABLED",start.elapsed().as_millis(),telemetry::placement(&target)).map_err(|e|e.to_string())
}
fn watch_motion(
    client: &mut Client,
    target: Position,
    input: &Receiver<Result<String, String>>,
    out: &mut impl Write,
    budget: Duration,
) -> Result<Option<String>, String> {
    writeln!(out,"Move command sent. Watch the board; when finished type done. Type stop to request STOP immediately. Waiting is quiet.").map_err(|e|e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    let start = Instant::now();
    let deadline = start + budget;
    let mut matches = 0;
    loop {
        match input.try_recv() {
            Ok(Ok(line)) => {
                send(client, &Command::stop_auto_move(), out)?;
                if line.trim() == "done" {
                    verify_after_done(client, target, out, start)?;
                    return Ok(None);
                }
                writeln!(
                    out,
                    "motion=STOP_REQUESTED physical_stop=UNVERIFIED; inspect board"
                )
                .map_err(|e| e.to_string())?;
                return if matches!(line.trim(), "stop" | "cancel") {
                    Ok(None)
                } else {
                    Ok(Some(line))
                };
            }
            Ok(Err(e)) => {
                let _ = send(client, &Command::stop_auto_move(), out);
                return Err(e);
            }
            Err(TryRecvError::Disconnected) => {
                send(client, &Command::stop_auto_move(), out)?;
                return Ok(Some("quit".into()));
            }
            Err(TryRecvError::Empty) => {}
        }
        if Instant::now() >= deadline {
            let stop = send(client, &Command::stop_auto_move(), out);
            return Err(format!(
                "motion timeout after {}s; STOP result={stop:?}; inspect board, no retry",
                budget.as_secs()
            ));
        }
        if let Some(event)=client.next_observed(deadline.saturating_duration_since(Instant::now()).min(Duration::from_millis(25))).map_err(|e|format!("motion observation failed: {e:?}; native cleanup requests STOP, physical stop unverified"))? {
            match event.event {
                Ok(BoardEvent::PositionChanged(p))=>{
                    matches=if p==target{matches+1}else{0};
                    if matches>=3 {
                        send(client,&Command::stop_auto_move(),out)?;
                        writeln!(out,"motion=TARGET_OBSERVED consecutive_reports=3 elapsed_ms={} placement={} physical_behavior=HUMAN_CHECK_REQUIRED motion_permission=DISABLED",start.elapsed().as_millis(),telemetry::placement(&p)).map_err(|e|e.to_string())?;
                        return Ok(None);
                    }
                }
                Err(reason) if event.source==crate::ipc::POSITION_UUID=>{
                    let _=send(client,&Command::stop_auto_move(),out);
                    return Err(format!("motion sensor incomplete: {reason}; STOP requested; inspect board"));
                }
                _=>{}
            }
        }
    }
}
pub fn execute(
    line: &str,
    guard: &mut Guard,
    client: &mut Client,
    streaming: &mut bool,
    input: &Receiver<Result<String, String>>,
    out: &mut impl Write,
    timeout: Duration,
) -> Result<(bool, Option<String>), String> {
    let line = line.trim();
    if line == "motion-enable" {
        crate::m3_session::execute("telemetry", client, streaming, input, out, timeout)?;
        let Report::Capabilities { mtu: Some(mtu), .. } = client
            .request(Operation::Capabilities)
            .map_err(|e| format!("M4 MTU: {e:?}"))?
        else {
            return Err("measured MTU required".into());
        };
        if mtu < 38 {
            return Err("MTU below 38; motion blocked".into());
        }
        guard.enable();
        writeln!(out,"motion_permission=ENABLED for_one_confirmed_target; clear travel area, verify physical pieces/batteries and source/target; Normal mode only").map_err(|e|e.to_string())?;
    } else if let Some(fen) = line.strip_prefix("packet-fen ") {
        let target = TargetFen::parse(fen.trim_matches('"')).map_err(str::to_owned)?;
        writeln!(
            out,
            "OFFLINE target={} length=35 kind=WithoutResponse mode=Normal bytes={:02x?} writes=0",
            target.fen,
            target.packet().bytes()
        )
        .map_err(|e| e.to_string())?;
    } else if let Some(fen) = line.strip_prefix("set-fen ") {
        let target = TargetFen::parse(fen.trim_matches('"')).map_err(str::to_owned)?;
        if !*streaming {
            return Err("motion disabled; use motion-enable first".into());
        }
        let source = fresh(client)?;
        match Plan::new(source, target.position).and_then(|p| guard.preview(p, Instant::now())) {
            Err(reason) => {
                guard.reset();
                writeln!(out, "target=REFUSED reason={reason}; nothing sent")
                    .map_err(|e| e.to_string())?;
            }
            Ok(()) => {
                writeln!(out,"source_placement={}\ntarget_fen={}\nmode=Normal length=35 kind=WithoutResponse bytes={:02x?}\nConfirm physical arrangement and clear travel area. Type confirm within 30s, or cancel. No motor write yet.",telemetry::placement(&source),target.fen,target.packet().bytes()).map_err(|e|e.to_string())?;
            }
        }
    } else if line == "confirm" {
        if !*streaming {
            return Err("no telemetry or pending target; nothing sent".into());
        }
        let source = fresh(client)?;
        match guard.confirm(source, Instant::now()) {
            Err(reason) => {
                writeln!(out, "confirmation=REFUSED reason={reason}").map_err(|e| e.to_string())?
            }
            Ok(plan) => {
                client
                    .request(Operation::Arm)
                    .map_err(|e| format!("native arm failed: {e:?}"))?;
                send(client, &plan.command(), out)?;
                // Do not count positions buffered before the actual motor write completed.
                discard_background(client)?;
                return watch_motion(
                    client,
                    plan.target,
                    input,
                    out,
                    timeout.min(Duration::from_secs(60)),
                )
                .map(|next| (true, next));
            }
        }
    } else if line == "stop" {
        guard.reset();
        if *streaming {
            send(client, &Command::stop_auto_move(), out)?;
            writeln!(
                out,
                "motion=STOP_REQUESTED physical_stop=UNVERIFIED; inspect board"
            )
            .map_err(|e| e.to_string())?;
        } else {
            writeln!(
                out,
                "STOP unavailable before telemetry initialization; no motion has been armed"
            )
            .map_err(|e| e.to_string())?;
        }
    } else if line == "cancel" {
        guard.reset();
        writeln!(out, "pending_target=CANCELLED motion_permission=DISABLED")
            .map_err(|e| e.to_string())?;
    } else {
        return Ok((false, None));
    }
    Ok((true, None))
}

/// Guided fixed-source test used by move-board.sh. Quiet while waiting for input.
pub fn guided(
    client: &mut Client,
    input: &Receiver<Result<String, String>>,
    out: &mut impl Write,
    returning: bool,
    timeout: Duration,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), String> {
    let e2 = "4k3/8/8/8/8/8/4P3/4K3";
    let e7 = "4k3/4P3/8/8/8/8/8/4K3";
    let (source, target) = if returning { (e7, e2) } else { (e2, e7) };
    let expected = TargetFen::parse(&format!("{source} w - - 0 1")).unwrap();
    let target = TargetFen::parse(&format!("{target} w - - 0 1")).unwrap();
    let mut streaming = false;
    crate::m3_session::execute("telemetry", client, &mut streaming, input, out, timeout)?;
    let Report::Capabilities { mtu: Some(mtu), .. } = client
        .request(Operation::Capabilities)
        .map_err(|e| format!("MTU: {e:?}"))?
    else {
        return Err("MTU unknown; motion blocked".into());
    };
    if mtu < 38 {
        return Err("MTU insufficient; motion blocked".into());
    }
    let sensed = fresh(client)?;
    if sensed != expected.position {
        return Err(format!(
            "source mismatch: expected={source} sensed={}; no motor write",
            telemetry::placement(&sensed)
        ));
    }
    let plan = Plan::new(sensed, target.position).map_err(str::to_owned)?;
    let mut guard = Guard::default();
    guard.enable();
    guard.preview(plan, Instant::now()).map_err(str::to_owned)?;
    writeln!(out,"\nSOURCE: {source}\nTARGET: {}\nMTU: {mtu}; 35 bytes; WithoutResponse; Normal\nBytes: {:02x?}\nClear travel area and verify physical pieces.\nType confirm within 30 seconds to move; cancel to exit. When movement finishes type done for final verification. During movement type stop to request STOP.",target.fen,plan.command().bytes()).map_err(|e|e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("guided session interrupted; no automatic retry".into());
        }
        if Instant::now() >= deadline {
            return Err("confirmation expired; nothing sent".into());
        }
        // Process bounded background traffic silently so the input stays readable.
        discard_background(client)?;
        match input.recv_timeout(Duration::from_millis(25)) {
            Ok(Ok(line)) if line.trim() == "confirm" => {
                let plan = guard
                    .confirm(fresh(client)?, Instant::now())
                    .map_err(str::to_owned)?;
                client
                    .request(Operation::Arm)
                    .map_err(|e| format!("native arm: {e:?}"))?;
                send(client, &plan.command(), out)?;
                discard_background(client)?;
                watch_motion(
                    client,
                    plan.target,
                    input,
                    out,
                    timeout.min(Duration::from_secs(60)),
                )?;
                return Ok(());
            }
            Ok(Ok(line)) if matches!(line.trim(), "cancel" | "quit" | "stop") => {
                writeln!(out, "move=CANCELLED motor_writes=0").map_err(|e| e.to_string())?;
                return Ok(());
            }
            Ok(Ok(_)) => {
                writeln!(out, "Type confirm to move, or cancel to exit.")
                    .map_err(|e| e.to_string())?;
            }
            Ok(Err(e)) => return Err(e),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}
