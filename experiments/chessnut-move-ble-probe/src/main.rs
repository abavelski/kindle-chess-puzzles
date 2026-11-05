use chessnut_ble_probe::TargetFen;
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "preflight" => {
            use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
            use std::sync::{atomic::AtomicBool, Arc};
            let cancelled = Arc::new(AtomicBool::new(false));
            let mut handlers = Vec::new();
            for signal in [SIGINT, SIGTERM, SIGHUP] {
                handlers.push(
                    signal_hook::flag::register(signal, Arc::clone(&cancelled))
                        .map_err(|error| error.to_string())?,
                );
            }
            let executable = std::env::current_exe().map_err(|error| error.to_string())?;
            let directory = executable.parent().ok_or("missing executable directory")?;
            let worker = directory.join("kindlebt-worker");
            let mut process = std::process::Command::new(worker);
            process
                .arg("--preflight")
                .arg(directory.join("libkindlebt.so"))
                .arg(directory.join("required-libs.txt"));
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            println!(
                "timestamp_unix={timestamp} phase=M1 command=preflight build={} protocol=1.0.2",
                env!("CARGO_PKG_VERSION")
            );
            let result = chessnut_ble_probe::preflight::run_worker(
                &mut process,
                std::time::Duration::from_secs(15),
                &cancelled,
            );
            for handler in handlers {
                signal_hook::low_level::unregister(handler);
            }
            result
        }
        [command, option, address] if matches!(command.as_str(), "session" | "read-once" | "subscribe-once" | "position-once" | "move-once" | "return-once") && option == "--address" => {
            use std::{io::{self, BufRead, Read}, sync::{atomic::AtomicBool, mpsc, Arc}, time::Duration};
            if !chessnut_ble_probe::session::valid_address(address) {
                return Err("invalid Bluetooth address; expected six hexadecimal octets (address not logged)".into());
            }
            let cancelled = Arc::new(AtomicBool::new(false));
            let mut handlers = Vec::new();
            for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM, signal_hook::consts::SIGHUP] {
                handlers.push(signal_hook::flag::register(signal, Arc::clone(&cancelled)).map_err(|e|e.to_string())?);
            }
            let executable = std::env::current_exe().map_err(|e|e.to_string())?;
            let directory = executable.parent().ok_or("missing executable directory")?;
            let mut worker = std::process::Command::new(directory.join("kindlebt-worker"));
            worker.arg("--ipc").arg(directory.join("libkindlebt.so")).arg(directory.join("required-libs.txt"));
            let result = if !matches!(command.as_str(), "session" | "move-once" | "return-once") {
                use chessnut_ble_probe::session::ReadMode;
                let mode = match command.as_str() {
                    "subscribe-once" => ReadMode::Subscriptions,
                    "position-once" => ReadMode::Position,
                    _ => ReadMode::Full,
                };
                chessnut_ble_probe::session::run_staged_once(&mut worker,address,&mut io::stdout().lock(),cancelled,Duration::from_secs(45),Duration::from_secs(55),(mode,20))
            } else {
            let (sender, input) = mpsc::sync_channel(8);
            std::thread::spawn(move || {
                let mut stdin = io::stdin().lock();
                loop {
                    let mut line = Vec::new();
                    match stdin.by_ref().take(257).read_until(b'\n', &mut line) {
                        Ok(0) => break,
                        Ok(_) if line.len() <= 256 => {
                            let line=String::from_utf8(line).map_err(|_|"non-UTF8 command".into());
                            if sender.send(line).is_err() { break; }
                        }
                        _ => { let _=sender.send(Err("command input invalid or exceeds 256 bytes".into())); break; }
                    }
                }
            });
            if command=="session" {
            chessnut_ble_probe::session::run_session(&mut worker,address,input,&mut io::stdout().lock(),cancelled,Duration::from_secs(45),Duration::from_secs(55))
            } else {
                let mut output=GuidedOutput::new().map_err(|e|e.to_string())?;
                let result=chessnut_ble_probe::session::run_guided_move(&mut worker,address,input,&mut output,cancelled,(command=="return-once",Duration::from_secs(45),Duration::from_secs(55)));
                use std::io::Write;
                let _=writeln!(output,"GUIDED_SESSION_RESULT={}{}",if result.is_ok(){"CLOSED"}else{"FAIL"},result.as_ref().err().map_or_else(String::new,|e|format!(" reason={e}")));
                result
            }
            };
            for handler in handlers { signal_hook::low_level::unregister(handler); }
            result
        }
        [command, fen] if command == "packet-fen" => {
            let target = TargetFen::parse(fen).map_err(str::to_owned)?;
            let packet = target.packet();
            println!("Target FEN: {}", target.fen);
            println!(
                "{} bytes; {:?}; Normal; OFFLINE ONLY",
                packet.bytes().len(),
                packet.write_kind()
            );
            println!(
                "{}",
                packet
                    .bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            Ok(())
        }
        [command] if command == "--help" || command == "help" => {
            println!("chessnut-ble-probe packet-fen '<six-field FEN>'\nMilestone 4: preflight (no connection), packet-fen (offline), session/read-once/subscribe-once/position-once --address XX:XX:XX:XX:XX:XX. Motion defaults disabled. Guided move-once/return-once require explicit confirmation. Interactive session supports motion-enable, set-fen, confirm and stop. No startup or read-script motor writes.");
            Ok(())
        }
        _ => Err(
            "UNSUPPORTED: Milestone 3 provides preflight, session --address <private BLE address>, packet-fen '<six-field FEN>' and --help"
                .into(),
        ),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

/// Optional private log mirror for guided scripts; no background screen output.
struct GuidedOutput {
    screen: std::io::Stdout,
    log: Option<std::fs::File>,
}
impl GuidedOutput {
    fn new() -> std::io::Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let log = std::env::var_os("PROBE_SESSION_LOG")
            .map(|path| {
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(path)
            })
            .transpose()?;
        Ok(Self {
            screen: std::io::stdout(),
            log,
        })
    }
}
impl std::io::Write for GuidedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.screen.write_all(bytes)?;
        if let Some(log) = &mut self.log {
            log.write_all(bytes)?;
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.screen.flush()?;
        if let Some(log) = &mut self.log {
            log.flush()?;
        }
        Ok(())
    }
}
