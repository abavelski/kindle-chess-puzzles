//! Task 06 Kindle loop with collection discovery and durable progress.
//!
//! Filesystem paths live in kindle-platform and are configurable through
//! KINDLE_CHESS_PUZZLE_DIR / KINDLE_CHESS_PROGRESS_FILE. Exit through the
//! controlling shell with Ctrl-C/SIGTERM; lifecycle ownership remains Task 07.

#![forbid(unsafe_code)]

use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, Effect, Progress, PuzzleCollection,
};
use chess_render::{render, DisplayMetrics};
use kindle_platform::{
    task04_scribe_transform, DiscoveredCollection, FingerInput, KindleDisplay, KindleStorage,
    ProgressStore, StoragePaths, TapPolicy, SCRIBE_DPI,
};
use std::collections::VecDeque;

const BUNDLED_PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const BUNDLED_FALLBACK_KEY: &str = "bundled-examples.json";

fn main() {
    if let Err(error) = run() {
        eprintln!("kindle-chess: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let storage = KindleStorage::new(StoragePaths::from_env());
    let discovered = storage.prepare_collections(BUNDLED_PUZZLES)?;
    let (mut progress_store, progress_load) =
        ProgressStore::open(storage.paths().progress_file.clone());
    let loaded_progress = progress_load.progress.clone();

    let (active_key, collection, persistence_enabled, collection_warning) =
        select_initial_collection(&storage, &discovered, &progress_load.progress)?;
    let mut app = AppState::new(
        ActiveCollection::from_collection(active_key, collection),
        progress_load.progress,
    );
    app.set_collection_entries(
        discovered
            .iter()
            .map(DiscoveredCollection::as_core_entry)
            .collect(),
    );

    let mut startup_message = progress_load.warning;
    if let Some(warning) = collection_warning {
        append_message(&mut startup_message, warning);
    }

    if persistence_enabled && app.progress() != &loaded_progress {
        progress_store.mark_dirty();
    }
    if persistence_enabled && progress_store.dirty() {
        if let Err(error) = progress_store.retry_if_dirty(app.progress()) {
            append_message(
                &mut startup_message,
                format!("Progress warning: {error}"),
            );
        }
    }
    if let Some(message) = startup_message {
        app.dispatch(Action::SetTransientMessage(Some(message)));
    }

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
        "kindle-chess: puzzles={} progress={} active={} persistent={}",
        storage.paths().puzzle_dir.display(),
        progress_store.path().display(),
        app.active_collection().key(),
        persistence_enabled
    );
    for entry in &discovered {
        match &entry.error {
            Some(error) => eprintln!(
                "kindle-chess: collection {} INVALID: {}",
                entry.filename, error
            ),
            None => eprintln!(
                "kindle-chess: collection {} -> {}",
                entry.filename, entry.label
            ),
        }
    }
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
    eprintln!("kindle-chess: tap FILES in the header to switch collections");

    loop {
        let output = render(&app, metrics)?;
        display.present(&output.frame)?;

        let (x, y) = input.next_tap()?;
        if let Some(target) = output.layout.hit_test_app(x, y, &app) {
            eprintln!("kindle-chess: tap ({x},{y}) -> {target:?}");
            let effects = app.dispatch(target.into_action());
            apply_effects(
                &mut app,
                &storage,
                &mut progress_store,
                persistence_enabled,
                effects,
            );
        } else {
            eprintln!("kindle-chess: tap ({x},{y}) -> no target");
            retry_dirty_progress(&mut app, &mut progress_store, persistence_enabled);
        }
    }
}

fn select_initial_collection(
    storage: &KindleStorage,
    discovered: &[DiscoveredCollection],
    progress: &Progress,
) -> Result<(String, PuzzleCollection, bool, Option<String>), Box<dyn std::error::Error>> {
    let mut valid = discovered
        .iter()
        .filter(|entry| entry.is_valid())
        .map(|entry| entry.filename.as_str())
        .collect::<Vec<_>>();

    if let Some(preferred) = progress.active_file.as_deref() {
        if let Some(index) = valid.iter().position(|filename| *filename == preferred) {
            valid.swap(0, index);
        }
    }

    let mut load_errors = Vec::new();
    for filename in valid {
        match storage.load_collection(filename) {
            Ok(collection) => {
                let warning = progress
                    .active_file
                    .as_deref()
                    .filter(|remembered| *remembered != filename)
                    .map(|remembered| {
                        format!(
                            "Remembered collection {remembered} is missing or invalid; using {filename}."
                        )
                    });
                return Ok((filename.to_owned(), collection, true, warning));
            }
            Err(error) => load_errors.push(error.to_string()),
        }
    }

    let collection = parse_puzzle_file(BUNDLED_PUZZLES)?;
    let mut warning =
        "No valid puzzle collection could be loaded. Bundled examples are shown in memory; existing matching files were not overwritten.".to_owned();
    if !load_errors.is_empty() {
        warning.push(' ');
        warning.push_str(&load_errors.join(" | "));
    }
    Ok((
        BUNDLED_FALLBACK_KEY.to_owned(),
        collection,
        false,
        Some(warning),
    ))
}

fn apply_effects(
    app: &mut AppState,
    storage: &KindleStorage,
    progress_store: &mut ProgressStore,
    persistence_enabled: bool,
    effects: Vec<Effect>,
) {
    let mut queue = VecDeque::from(effects);

    while let Some(effect) = queue.pop_front() {
        match effect {
            Effect::ProgressChanged => {
                if persistence_enabled {
                    progress_store.mark_dirty();
                }
            }
            Effect::CollectionRequested(filename) => match storage.load_collection(&filename) {
                Ok(collection) => {
                    queue.extend(app.dispatch(Action::ActivateCollection(
                        ActiveCollection::from_collection(filename, collection),
                    )));
                }
                Err(error) => {
                    app.dispatch(Action::CloseCollectionPicker);
                    app.dispatch(Action::SetTransientMessage(Some(format!(
                        "Collection error: {error}"
                    ))));
                }
            },
        }
    }

    retry_dirty_progress(app, progress_store, persistence_enabled);
}

fn retry_dirty_progress(
    app: &mut AppState,
    progress_store: &mut ProgressStore,
    persistence_enabled: bool,
) {
    if !persistence_enabled || !progress_store.dirty() {
        return;
    }

    match progress_store.retry_if_dirty(app.progress()) {
        Ok(_) => {
            if app
                .transient_message()
                .is_some_and(|message| message.starts_with("Progress warning:"))
            {
                app.dispatch(Action::SetTransientMessage(None));
            }
        }
        Err(error) => {
            app.dispatch(Action::SetTransientMessage(Some(format!(
                "Progress warning: {error}"
            ))));
        }
    }
}

fn append_message(message: &mut Option<String>, addition: String) {
    match message {
        Some(existing) => {
            existing.push(' ');
            existing.push_str(&addition);
        }
        None => *message = Some(addition),
    }
}
