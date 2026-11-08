//! kindle-puzzles Kindle entrypoint.
#![forbid(unsafe_code)]
fn main() {
    if let Err(error) = kindle_app::products::run_puzzles() {
        eprintln!("kindle-puzzles: {error}");
        std::process::exit(1);
    }
}
