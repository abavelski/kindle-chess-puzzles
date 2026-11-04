use chessnut_ble_probe::TargetFen;
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
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
            println!("chessnut-ble-probe packet-fen '<six-field FEN>'\nMilestone 0: offline encoding only. Native preflight/session commands are UNSUPPORTED.");
            Ok(())
        }
        _ => Err(
            "UNSUPPORTED: Milestone 0 provides only packet-fen '<six-field FEN>' and --help".into(),
        ),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
