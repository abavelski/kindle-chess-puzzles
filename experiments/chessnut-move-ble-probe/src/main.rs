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
            println!("chessnut-ble-probe packet-fen '<six-field FEN>'\nMilestone 1: preflight (no connection) or offline packet-fen. Live session commands are UNSUPPORTED.");
            Ok(())
        }
        _ => Err(
            "UNSUPPORTED: Milestone 1 provides preflight, packet-fen '<six-field FEN>' and --help"
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
