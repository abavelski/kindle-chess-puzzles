//! kindle-game-review Kindle entrypoint.
#![forbid(unsafe_code)]
fn main() {
    if let Err(error) = kindle_app::products::run_review() {
        eprintln!("kindle-game-review: {error}");
        std::process::exit(1);
    }
}
