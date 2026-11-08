//! Kindle collection/progress loop with pixel-verified partial presentation.
//! Launch through scripts/kindle_launch.sh for single-instance signal cleanup.

#![forbid(unsafe_code)]

use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, Effect, Progress, PuzzleCollection,
};
use chess_render::HitTarget;
use kindle_app::{
    DiscoveredCollection, KindleStorage, ProgressStore, ReviewResumeStore, SettingsStore,
    StoragePaths,
};
#[cfg(test)]
use kindle_platform::{DeviceEvent, PowerEvent};
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
    let storage = KindleStorage::new(StoragePaths::from_env());
    let discovered = storage.prepare_collections(BUNDLED_PUZZLES)?;
    let review_library = storage.discover_review_library()?;
    let (mut progress_store, progress_load) =
        ProgressStore::open(storage.paths().progress_file.clone());
    let (settings_store, settings_load) =
        SettingsStore::open(storage.paths().settings_file.clone());
    let (review_store, review_load) = ReviewResumeStore::open(&storage.paths().review_resume_file);
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
    if let Some(warning) = review_load.warning {
        append_message(&mut startup_message, warning);
    }
    if let Some(resume) = review_load.resume {
        if let Some(warning) = app.restore_review_resume(&resume) {
            append_message(&mut startup_message, warning);
        }
    }
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
    kindle_app::runtime::run(
        CombinedRuntime {
            app,
            storage,
            progress_store,
            settings_store,
            review_store,
            persistence_enabled,
            board_service: kindle_app::chessnut::BoardService::new(),
        },
        process_start,
    )
}
#[cfg(test)]
use kindle_app::runtime::awake_tap;

struct CombinedRuntime {
    app: AppState,
    storage: KindleStorage,
    progress_store: ProgressStore,
    settings_store: SettingsStore,
    review_store: ReviewResumeStore,
    persistence_enabled: bool,
    board_service: kindle_app::chessnut::BoardService,
}
impl CombinedRuntime {
    fn effects(&mut self, effects: Vec<Effect>) -> bool {
        let effects = route_board_effects(&mut self.app, effects, |intent| {
            self.board_service.send(intent)
        });
        apply_effects(
            &mut self.app,
            &self.storage,
            &mut self.progress_store,
            &mut self.settings_store,
            &mut self.review_store,
            self.persistence_enabled,
            effects,
        )
    }
}
impl kindle_app::runtime::Application for CombinedRuntime {
    fn screen(&self) -> &impl chess_render::ScreenState {
        &self.app
    }
    fn poll(&mut self) -> kindle_app::runtime::Poll {
        let mut changed = false;
        loop {
            match self.board_service.poll() {
                Ok(Some(update)) => {
                    let action = match update {
                        kindle_app::chessnut::BoardUpdate::Status(status) => {
                            Action::BoardStatus(status)
                        }
                        kindle_app::chessnut::BoardUpdate::Position(position) => {
                            Action::BoardPosition(position)
                        }
                    };
                    let effects = self.app.dispatch(action);
                    if self.effects(effects) {
                        return kindle_app::runtime::Poll::Exit;
                    }
                    changed = true;
                }
                Ok(None) => break,
                Err(reason) => {
                    let status = chess_core::BoardSessionStatus::Error(reason);
                    if self.app.board_status() != &status {
                        self.app.dispatch(Action::BoardStatus(status));
                        changed = true;
                    }
                    break;
                }
            }
        }
        if changed {
            kindle_app::runtime::Poll::Changed
        } else {
            kindle_app::runtime::Poll::Idle
        }
    }
    fn tap(&mut self, target: HitTarget) -> bool {
        let effects = self
            .app
            .dispatch(target.into_action().expect("non-refresh app action"));
        self.effects(effects)
    }
    fn pause_input(&mut self) {
        self.app.dispatch(Action::PauseBoardInput);
    }
    fn flush(&mut self) {
        retry_dirty_state(
            &mut self.app,
            &mut self.progress_store,
            &mut self.settings_store,
            &mut self.review_store,
            self.persistence_enabled,
        );
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

fn route_board_effects(
    app: &mut AppState,
    effects: Vec<Effect>,
    mut send: impl FnMut(chess_core::BoardIntent) -> Result<(), String>,
) -> Vec<Effect> {
    effects
        .into_iter()
        .filter_map(|effect| {
            if let Effect::Board(intent) = effect {
                if let Err(reason) = send(intent) {
                    app.dispatch(Action::BoardStatus(chess_core::BoardSessionStatus::Error(
                        reason,
                    )));
                }
                None
            } else {
                Some(effect)
            }
        })
        .collect()
}

fn apply_effects(
    app: &mut AppState,
    storage: &KindleStorage,
    progress_store: &mut ProgressStore,
    settings_store: &mut SettingsStore,
    review_store: &mut ReviewResumeStore,
    persistence_enabled: bool,
    effects: Vec<Effect>,
) -> bool {
    let mut queue = VecDeque::from(effects);
    let mut exit_requested = false;

    while let Some(effect) = queue.pop_front() {
        match effect {
            Effect::Board(_) => {} // Routed to BoardService by the runtime before persistence.
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

    retry_dirty_state(
        app,
        progress_store,
        settings_store,
        review_store,
        persistence_enabled,
    );
    exit_requested
}

fn retry_dirty_state(
    app: &mut AppState,
    progress_store: &mut ProgressStore,
    settings_store: &mut SettingsStore,
    review_store: &mut ReviewResumeStore,
    persistence_enabled: bool,
) {
    let mut warning = None;

    if app.workspace() == chess_core::Workspace::Review {
        if let Some(resume) = app.review_resume() {
            review_store.observe(resume);
        }
    }
    if let Err(error) = review_store.retry_if_dirty() {
        append_message(&mut warning, format!("Review resume warning: {error}"));
    }

    if persistence_enabled && progress_store.dirty() {
        if let Err(error) = progress_store.retry_if_dirty(app.progress()) {
            append_message(&mut warning, format!("Progress warning: {error}"));
        }
    }

    if settings_store.dirty() {
        if let Err(error) = settings_store.retry_if_dirty(app.persisted_settings()) {
            append_message(&mut warning, format!("Settings warning: {error}"));
        }
    }

    if let Some(warning) = warning {
        app.dispatch(Action::SetTransientMessage(Some(warning)));
    } else if app.transient_message().is_some_and(|message| {
        message.starts_with("Progress warning:")
            || message.starts_with("Settings warning:")
            || message.starts_with("Review resume warning:")
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
    fn board_originated_reply_is_routed_once_and_progress_survives_transport_failure() {
        for fail in [false, true] {
            let mut collection =
                parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap();
            collection.puzzles[1].solution.truncate(2);
            let mut app = AppState::new(
                ActiveCollection::from_collection("puzzles.json", collection),
                Progress::new(),
            );
            app.dispatch(Action::NextPuzzle);
            app.dispatch(Action::BoardStatus(
                chess_core::BoardSessionStatus::Connected,
            ));
            let baseline = app.board().placement();
            app.dispatch(Action::SendBoardPosition);
            app.dispatch(Action::BoardStatus(
                chess_core::BoardSessionStatus::Connected,
            ));
            app.dispatch(Action::BoardPosition(baseline));
            let mut physical = app.board().clone();
            physical.apply_uci_move(chess_core::parse_uci_move("e2e6").unwrap());
            let effects = app.dispatch(Action::BoardPosition(physical.placement()));
            let target = app.board().placement();
            let mut submitted = Vec::new();
            let remaining = route_board_effects(&mut app, effects, |intent| {
                submitted.push(intent);
                if fail {
                    Err("worker unavailable".into())
                } else {
                    Ok(())
                }
            });
            assert_eq!(
                submitted,
                vec![chess_core::BoardIntent::Send(target.clone())]
            );
            assert_eq!(remaining, vec![Effect::ProgressChanged]);
            assert_eq!(app.solution_ply(), 2);
            assert_eq!(app.board().placement(), target);
            assert_eq!(app.feedback(), chess_core::SolutionFeedback::Complete);
            if fail {
                assert!(matches!(
                    app.board_status(),
                    chess_core::BoardSessionStatus::Error(_)
                ));
                assert_eq!(app.board_input_status(), chess_core::BoardInputStatus::Off);
            }
        }
    }

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
        let (mut review_store, _) = ReviewResumeStore::open(&storage.paths().review_resume_file);

        assert!(app.dispatch(Action::ToggleWorkspace).is_empty());
        assert!(app.dispatch(Action::ReviewNext).is_empty());
        assert!(app.dispatch(Action::OpenReviewGamePicker).is_empty());
        let effects = app.dispatch(Action::SelectReviewGame(0));
        assert!(!apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
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
    fn runtime_saves_both_profiles_when_settings_change_in_review() {
        let root =
            std::env::temp_dir().join(format!("kcp-workspace-settings-{}", std::process::id()));
        let storage = KindleStorage::new(StoragePaths::new(
            root.join("puzzles"),
            root.join("state/progress.json"),
        ));
        let (mut progress_store, _) = ProgressStore::open(storage.paths().progress_file.clone());
        let (mut settings_store, _) = SettingsStore::open(storage.paths().settings_file.clone());
        let (mut review_store, _) = ReviewResumeStore::open(&storage.paths().review_resume_file);
        let mut app = AppState::new(
            ActiveCollection::from_collection(
                "puzzles.json",
                parse_puzzle_file(BUNDLED_PUZZLES).unwrap(),
            ),
            Progress::new(),
        );
        let games = chess_core::parse_review_file(include_bytes!(
            "../../../tests/fixtures/game-review/valid-standard.json"
        ))
        .unwrap();
        app.set_review_games(vec![chess_core::ReviewGameEntry::new(
            "games.json",
            games.games[0].clone(),
        )]);
        app.dispatch(Action::ToggleWorkspace);
        let effects = app.dispatch(Action::ToggleBoardSizeSetting);
        apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects,
        );
        let (_, load) = SettingsStore::open(storage.paths().settings_file.clone());
        assert!(load.warning.is_none());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .small_board());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .small_board());
        for action in [Action::ToggleLockSetting, Action::ToggleFlipSetting] {
            let effects = app.dispatch(action);
            apply_effects(
                &mut app,
                &storage,
                &mut progress_store,
                &mut settings_store,
                &mut review_store,
                true,
                effects,
            );
        }
        app.dispatch(Action::ToggleWorkspace);
        let effects = app.dispatch(Action::ToggleNotesSetting);
        apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects,
        );
        let (_, load) = SettingsStore::open(storage.paths().settings_file.clone());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .show_notes_button());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .show_notes_button());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .small_board());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .show_lock_button());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .show_flip_button());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .show_lock_button());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .show_flip_button());
        let effects = app.dispatch(Action::ToggleFlipSetting);
        apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects,
        );
        let (_, load) = SettingsStore::open(storage.paths().settings_file.clone());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .show_flip_button());
        assert!(load
            .settings
            .for_workspace(chess_core::Workspace::Puzzles)
            .show_lock_button());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .show_lock_button());
        assert!(!load
            .settings
            .for_workspace(chess_core::Workspace::Review)
            .show_flip_button());
        assert!(!progress_store.dirty());
        assert!(!storage.paths().progress_file.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runtime_restores_review_after_restart_and_flushes_latest_on_exit() {
        let root = std::env::temp_dir().join(format!("kcp-review-restart-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let storage = KindleStorage::new(StoragePaths::new(
            root.join("puzzles"),
            root.join("state/progress.json"),
        ));
        std::fs::create_dir_all(&storage.paths().review_dir).unwrap();
        let source = include_bytes!("../../../tests/fixtures/game-review/valid-standard.json");
        std::fs::write(storage.paths().review_dir.join("games.json"), source).unwrap();
        let make_app = || {
            let mut app = AppState::new(
                ActiveCollection::from_collection(
                    "puzzles.json",
                    parse_puzzle_file(BUNDLED_PUZZLES).unwrap(),
                ),
                Progress::new(),
            );
            app.set_review_games(storage.discover_review_library().unwrap().games);
            app
        };
        let mut app = make_app();
        let progress = app.progress().to_bytes().unwrap();
        std::fs::create_dir_all(storage.paths().progress_file.parent().unwrap()).unwrap();
        std::fs::write(&storage.paths().progress_file, &progress).unwrap();
        let (mut progress_store, _) = ProgressStore::open(&storage.paths().progress_file);
        let (mut settings_store, _) = SettingsStore::open(&storage.paths().settings_file);
        let (mut review_store, _) =
            kindle_app::ReviewResumeStore::open(&storage.paths().review_resume_file);
        app.dispatch(Action::ToggleWorkspace);
        app.dispatch(Action::ReviewNext);
        let effects = app.dispatch(Action::ReviewNext);
        apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects,
        );
        let state_dir = root.join("state");
        let backup = root.join("state-backup");
        std::fs::rename(&state_dir, &backup).unwrap();
        std::fs::write(&state_dir, b"temporarily unavailable").unwrap();
        let effects = app.dispatch(Action::ReviewNext);
        apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects,
        );
        assert!(review_store.dirty());
        assert!(app
            .transient_message()
            .unwrap()
            .starts_with("Review resume warning:"));
        let saved_board = app.board().clone();
        app.dispatch(Action::ToggleWorkspace);
        std::fs::remove_file(&state_dir).unwrap();
        std::fs::rename(&backup, &state_dir).unwrap();
        let effects = app.dispatch(Action::Exit);
        assert!(apply_effects(
            &mut app,
            &storage,
            &mut progress_store,
            &mut settings_store,
            &mut review_store,
            true,
            effects
        ));
        assert!(!review_store.dirty());
        let (_, loaded) = kindle_app::ReviewResumeStore::open(&storage.paths().review_resume_file);
        let mut restored = make_app();
        assert!(restored
            .restore_review_resume(&loaded.resume.unwrap())
            .is_none());
        assert_eq!(restored.workspace(), chess_core::Workspace::Puzzles);
        restored.dispatch(Action::ToggleWorkspace);
        assert_eq!(restored.board(), &saved_board);
        assert_eq!(
            std::fs::read(&storage.paths().progress_file).unwrap(),
            progress
        );
        assert_eq!(
            std::fs::read(storage.paths().review_dir.join("games.json")).unwrap(),
            source
        );
        assert!(!storage.paths().settings_file.exists());
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
        let (mut review_store, _) = ReviewResumeStore::open(&storage.paths().review_resume_file);
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
            &mut review_store,
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
