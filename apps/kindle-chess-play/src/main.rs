//! kindle-chess-play Kindle entrypoint.
#![forbid(unsafe_code)]
fn main() {
    eprintln!("KINDLE_CHESS_PRODUCT:play:v1");
    if let Err(error) = kindle_app::products::run_play() {
        eprintln!("kindle-chess-play: {error}");
        std::process::exit(1);
    }
}
