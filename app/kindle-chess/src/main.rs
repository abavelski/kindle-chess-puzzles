//! Single-collection Task 05 Kindle parity loop.
//!
//! This intentionally uses the bundled parity fixture collection and no durable storage.
//! Exit through the controlling shell with Ctrl-C/SIGTERM; lifecycle ownership
//! and signal cleanup policy are expanded in Task 07.

#![forbid(unsafe_code)]

use chess_core::{parse_puzzle_file, ActiveCollection, AppState, Progress};
use chess_render::{render, DisplayMetrics};
use kindle_platform::{task04_scribe_transform, FingerInput, KindleDisplay, TapPolicy, SCRIBE_DPI};

const BUNDLED_PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");

fn main() {
    if let Err(error) = run() {
        eprintln!("kindle-chess: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let collection = parse_puzzle_file(BUNDLED_PUZZLES)?;
    let mut app = AppState::new(
        ActiveCollection::from_collection("bundled-task05-parity.json", collection),
        Progress::new(),
    );

    let mut display = KindleDisplay::open()?;
    let display_state = display.state();
    let metrics = DisplayMetrics {
        width: display_state.width,
        height: display_state.height,
        dpi: SCRIBE_DPI,
    };
    let transform = task04_scribe_transform(metrics)?;
    let mut input = FingerInput::discover(transform, TapPolicy::scribe_default())?;

    eprintln!(
        "kindle-chess: FBInk {} at {}x{} stride={} bpp={} rotation={}",
        display.fbink_version()?,
        display_state.width,
        display_state.height,
        display_state.scanline_stride,
        display_state.bpp,
        display_state.rotation
    );
    for diagnostic in input.diagnostics() {
        eprintln!("kindle-chess: input candidate {diagnostic}");
    }
    eprintln!(
        "kindle-chess: selected /dev/input/{} ({})",
        input.selected().event_name,
        input.selected().name
    );
    eprintln!("kindle-chess: Ctrl-C or SIGTERM exits this Task 05 loop");

    loop {
        let output = render(&app, metrics)?;
        display.present(&output.frame)?;

        let (x, y) = input.next_tap()?;
        let promotion_open = app.pending_promotion().is_some();
        if let Some(target) = output.layout.hit_test(x, y, app.flipped(), promotion_open) {
            eprintln!("kindle-chess: tap ({x},{y}) -> {target:?}");
            let _effects = app.dispatch(target.into_action());
        } else {
            eprintln!("kindle-chess: tap ({x},{y}) -> no target");
        }
    }
}
