//! Kindle collection/progress loop with pixel-verified partial presentation.
//! Launch through scripts/kindle_launch.sh for single-instance signal cleanup.

#![forbid(unsafe_code)]

use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, Effect, Progress, PuzzleCollection,
};
use chess_render::{
    calculate_damage, compact_damage, draw_sleeping_overlay, render, DisplayMetrics, HitTarget,
};
use kindle_platform::{
    clean_regions_for_board_change, DeviceEvent, DiscoveredCollection, KindleDisplay,
    KindleStorage, PowerEvent, PowerEvents, ProgressStore, RefreshPolicy, ScribeInput,
    SettingsStore, StoragePaths, TapPolicy, SCRIBE_DPI,
};
use std::collections::VecDeque;
use std::time::Instant;

const BUNDLED_PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const BUNDLED_FALLBACK_KEY: &str = "bundled-examples.json";

type InitialCollection = (String, PuzzleCollection, bool, Option<String>);

fn main() {
    if let Err(error) = run() {
        eprintln!("kindle-chess: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let process_start = Instant::now();
    eprintln!("kindle-chess: timing process_start=0ms");
    let force_full = std::env::args().skip(1).any(|arg| arg == "--full-refresh");
    let storage = KindleStorage::new(StoragePaths::from_env());
    let discovered = storage.prepare_collections(BUNDLED_PUZZLES)?;
    let review_library = storage.discover_review_library()?;
    let (mut progress_store, progress_load) =
        ProgressStore::open(storage.paths().progress_file.clone());
    let (mut settings_store, settings_load) =
        SettingsStore::open(storage.paths().settings_file.clone());
    let loaded_progress = progress_load.progress.clone();

    let (active_key, collection, persistence_enabled, collection_warning) =
        select_initial_collection(&storage, &discovered, &progress_load.progress)?;
    let mut app = AppState::new_with_settings(
        ActiveCollection::from_collection(active_key, collection),
        progress_load.progress,
        settings_load.settings,
    );
    app.set_collection_entries(
        discovered
            .iter()
            .map(DiscoveredCollection::as_core_entry)
            .collect(),
    );
    app.set_review_games(review_library.games);
    app.set_review_file_errors(review_library.errors);

    let mut startup_message = progress_load.warning;
    if let Some(warning) = settings_load.warning {
        append_message(&mut startup_message, warning);
    }
    if let Some(warning) = collection_warning {
        append_message(&mut startup_message, warning);
    }

    if persistence_enabled && app.progress() != &loaded_progress {
        progress_store.mark_dirty();
    }
    if persistence_enabled && progress_store.dirty() {
        if let Err(error) = progress_store.retry_if_dirty(app.progress()) {
            append_message(&mut startup_message, format!("Progress warning: {error}"));
        }
    }
    if let Some(message) = startup_message {
        app.dispatch(Action::SetTransientMessage(Some(message)));
    }

    let mut display = KindleDisplay::open()?;
    eprintln!(
        "kindle-chess: timing fbink_initialized={}ms",
        process_start.elapsed().as_millis()
    );
    let display_state = display.state();
    let metrics = DisplayMetrics {
        width: display_state.width,
        height: display_state.height,
        dpi: SCRIBE_DPI,
    };
    let mut input = ScribeInput::discover(metrics, TapPolicy::scribe_default())?;
    input.take_exclusive()?;
    eprintln!("kindle-chess: exclusive finger input acquired before first frame");

    eprintln!(
        "kindle-chess: puzzles={} reviews={} progress={} active={} review_games={} persistent={}",
        storage.paths().puzzle_dir.display(),
        storage.paths().review_dir.display(),
        progress_store.path().display(),
        app.active_collection().key(),
        app.review_games().len(),
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
    for entry in app.review_games() {
        eprintln!(
            "kindle-chess: review game {}::{} -> {}",
            entry.key().collection_id(),
            entry.key().game_id(),
            entry.label().replace('\n', " | ")
        );
    }
    for error in app.review_file_errors() {
        eprintln!(
            "kindle-chess: review file {} INVALID: {}",
            error.collection_id(),
            error.error()
        );
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
    eprintln!(
        "kindle-chess: selected pen /dev/input/{} ({})",
        input.selected_pen().event_name,
        input.selected_pen().name
    );
    eprintln!("kindle-chess: tap FILES in the header to switch collections");

    let mut power = PowerEvents::open()?;
    let mut sleeping = false;
    let mut previous = None;
    let mut policy = RefreshPolicy::default();
    let mut touch_received: Option<Instant> = None;
    let mut clean_regions = Vec::new();
    let mut force_full_next = false;
    loop {
        let mut output = render(&app, metrics)?;
        if sleeping {
            draw_sleeping_overlay(&mut output.frame, output.layout, metrics);
        }
        output.damage = compact_damage(
            &calculate_damage(previous.as_ref(), &output.frame),
            output.layout,
        );
        let timing = display.present_damage(
            &output.frame,
            previous.as_ref(),
            &output.damage,
            &clean_regions,
            &mut policy,
            force_full || force_full_next,
        )?;
        force_full_next = false;
        if previous.is_none() {
            eprintln!(
                "kindle-chess: timing first_usable_frame={}ms",
                process_start.elapsed().as_millis()
            );
        }
        if timing.regions > 0 {
            let touch_to_submit = touch_received.map(|start| {
                start
                    .elapsed()
                    .saturating_sub(timing.complete)
                    .saturating_add(timing.submit)
            });
            eprintln!("kindle-chess: timing regions={} full={} submit={}ms complete={}ms recognized_touch_to_submit={:?}", timing.regions, timing.full, timing.submit.as_millis(), timing.complete.as_millis(), touch_to_submit);
        }
        previous = Some(output.frame.clone());
        clean_regions.clear();
        let event = input.next_event(&mut power)?;
        if matches!(event, DeviceEvent::Power(_)) {
            touch_received = None;
        }
        if event == DeviceEvent::Power(PowerEvent::Awake) {
            // Native wake can repaint any pixels while Xorg is briefly resumed.
            force_full_next = kindle_platform::complete_native_wake(&mut power)?;
        }
        let Some((x, y)) = awake_tap(event, &mut sleeping) else {
            continue;
        };
        touch_received = Some(Instant::now());
        let target = output.hit_test_app(x, y, &app);

        if let Some(target) = target {
            eprintln!("kindle-chess: tap ({x},{y}) -> {target:?}");
            if target == HitTarget::Refresh {
                force_full_next = true;
                eprintln!("kindle-chess: full-screen refresh requested");
                continue;
            }
            let board_before = app.board().clone();
            let flipped_before = app.flipped();
            let effects = app.dispatch(
                target
                    .into_action()
                    .expect("non-refresh hit targets map to app actions"),
            );
            if apply_effects(
                &mut app,
                &storage,
                &mut progress_store,
                &mut settings_store,
                persistence_enabled,
                effects,
            ) {
                eprintln!("kindle-chess: Exit requested; releasing display and input");
                return Ok(());
            }
            clean_regions = clean_regions_for_board_change(
                &board_before,
                app.board(),
                flipped_before,
                app.flipped(),
                output.layout,
            );
        } else {
            eprintln!("kindle-chess: tap ({x},{y}) -> no target");
            retry_dirty_state(
                &mut app,
                &mut progress_store,
                &mut settings_store,
                persistence_enabled,
            );
        }
    }
}

fn awake_tap(event: DeviceEvent, sleeping: &mut bool) -> Option<(u32, u32)> {
    match event {
        DeviceEvent::Power(PowerEvent::NativeWakeComplete) => None,
        DeviceEvent::Power(event) => {
            *sleeping = event == PowerEvent::Sleeping;
            eprintln!("kindle-chess: power {event:?}");
            None
        }
        DeviceEvent::Tap(_, _) if *sleeping => None,
        DeviceEvent::Tap(x, y) => Some((x, y)),
    }
}

fn select_initial_collection(
    storage: &KindleStorage,
    discovered: &[DiscoveredCollection],
    progress: &Progress,
) -> Result<InitialCollection, Box<dyn std::error::Error>> {
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
    settings_store: &mut SettingsStore,
    persistence_enabled: bool,
    effects: Vec<Effect>,
) -> bool {
    let mut queue = VecDeque::from(effects);
    let mut exit_requested = false;

    while let Some(effect) = queue.pop_front() {
        match effect {
            Effect::ExitRequested => exit_requested = true,
            Effect::ProgressChanged => {
                if persistence_enabled {
                    progress_store.mark_dirty();
                }
            }
            Effect::SettingsChanged => settings_store.mark_dirty(),
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
            Effect::ReviewGameRequested(key) => match storage.load_review_game(&key) {
                Ok(game) => {
                    queue.extend(app.dispatch(Action::ActivateReviewGame(key, Box::new(game))));
                }
                Err(error) => {
                    app.dispatch(Action::CloseReviewGamePicker);
                    app.dispatch(Action::SetTransientMessage(Some(format!(
                        "Review game error: {error}"
                    ))));
                }
            },
        }
    }

    retry_dirty_state(app, progress_store, settings_store, persistence_enabled);
    exit_requested
}

fn retry_dirty_state(
    app: &mut AppState,
    progress_store: &mut ProgressStore,
    settings_store: &mut SettingsStore,
    persistence_enabled: bool,
) {
    let mut warning = None;

    if persistence_enabled && progress_store.dirty() {
        if let Err(error) = progress_store.retry_if_dirty(app.progress()) {
            warning = Some(format!("Progress warning: {error}"));
        }
    }

    if settings_store.dirty() {
        if let Err(error) = settings_store.retry_if_dirty(app.settings()) {
            append_message(&mut warning, format!("Settings warning: {error}"));
        }
    }

    if let Some(warning) = warning {
        app.dispatch(Action::SetTransientMessage(Some(warning)));
    } else if app.transient_message().is_some_and(|message| {
        message.starts_with("Progress warning:") || message.starts_with("Settings warning:")
    }) {
        app.dispatch(Action::SetTransientMessage(None));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_wake_events_gate_taps_without_changing_puzzle_state() {
        let mut sleeping = false;
        assert_eq!(
            awake_tap(DeviceEvent::Tap(1, 2), &mut sleeping),
            Some((1, 2))
        );
        assert_eq!(
            awake_tap(DeviceEvent::Power(PowerEvent::Sleeping), &mut sleeping),
            None
        );
        assert!(sleeping);
        assert_eq!(awake_tap(DeviceEvent::Tap(1, 2), &mut sleeping), None);
        assert_eq!(
            awake_tap(DeviceEvent::Power(PowerEvent::Awake), &mut sleeping),
            None
        );
        assert!(!sleeping);
        assert_eq!(
            awake_tap(DeviceEvent::Tap(1, 2), &mut sleeping),
            Some((1, 2))
        );
    }

    #[test]
    fn review_runtime_keeps_puzzle_progress_bytes_stable() {
        const REVIEW: &[u8] =
            include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");

        let root = std::env::temp_dir().join(format!("kcp-review-runtime-{}", std::process::id()));
        let storage = KindleStorage::new(StoragePaths::new(
            root.join("puzzles"),
            root.join("state/progress.json"),
        ));
        std::fs::create_dir_all(&storage.paths().review_dir).unwrap();
        std::fs::write(storage.paths().review_dir.join("games.json"), REVIEW).unwrap();

        let library = storage.discover_review_library().unwrap();
        assert_eq!(library.games.len(), 1);
        assert!(library.errors.is_empty());

        let mut app = AppState::new(
            ActiveCollection::from_collection(
                "puzzles.json",
                parse_puzzle_file(BUNDLED_PUZZLES).unwrap(),
            ),
            Progress::new(),
        );
        app.set_review_games(library.games);
        app.set_review_file_errors(library.errors);

        let original = app.progress().to_bytes().unwrap();
        std::fs::create_dir_all(storage.paths().progress_file.parent().unwrap()).unwrap();
        std::fs::write(&storage.paths().progress_file, &original).unwrap();
        let (mut progress_store, load) = ProgressStore::open(storage.paths().progress_file.clone());
        assert_eq!(load.progress, *app.progress());
        let (mut settings_store, _) = SettingsStore::open(storage.paths().settings_file.clone());

        assert!(app.dispatch(Action::ToggleWorkspace).is_empty());
        assert!(app.dispatch(Action::ReviewNext).is_empty());
        assert!(app.dispatch(Action::OpenReviewGamePicker).is_empty());
        let effects = app.dispatch(Action::SelectReviewGame(0));
        assert!(!apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            true,
            effects,
        ));
        assert!(app.dispatch(Action::ReviewNext).is_empty());
        assert!(app.dispatch(Action::ToggleWorkspace).is_empty());

        assert!(!progress_store.dirty());
        assert_eq!(std::fs::read(progress_store.path()).unwrap(), original);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exit_flushes_dirty_progress_and_requests_normal_return() {
        let root = std::env::temp_dir().join(format!("kcp-exit-{}", std::process::id()));
        let storage = KindleStorage::new(StoragePaths::new(
            root.join("puzzles"),
            root.join("state/progress.json"),
        ));
        let (mut store, _) = ProgressStore::open(storage.paths().progress_file.clone());
        let (mut settings_store, _) = SettingsStore::open(storage.paths().settings_file.clone());
        let mut app = AppState::new(
            ActiveCollection::from_collection(
                "puzzles.json",
                parse_puzzle_file(BUNDLED_PUZZLES).unwrap(),
            ),
            Progress::new(),
        );
        store.mark_dirty();
        let effects = app.dispatch(Action::Exit);
        assert!(apply_effects(
            &mut app,
            &storage,
            &mut store,
            &mut settings_store,
            true,
            effects
        ));
        assert!(!store.dirty());
        assert_eq!(
            Progress::parse(&std::fs::read(store.path()).unwrap()).unwrap(),
            *app.progress()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
