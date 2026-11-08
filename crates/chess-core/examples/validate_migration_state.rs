//! Host-only parser for the one-time data migration tool.
use std::io::Read;

fn main() {
    let mut bytes = Vec::new();
    if let Err(error) = std::io::stdin().read_to_end(&mut bytes) {
        eprintln!("{error}");
        std::process::exit(1);
    }
    let result = match std::env::args().nth(1).as_deref() {
        Some("settings") => chess_core::Settings::parse(&bytes).map(|_| ()),
        Some("resume") => chess_core::ReviewResume::parse(&bytes).map(|_| ()),
        _ => Err("Expected settings or resume".into()),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
