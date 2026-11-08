use crate::{
    chessnut::{BoardService, BoardUpdate},
    runtime::{Application, Poll},
    startup::{select_initial_collection, BUNDLED_PUZZLES},
    DiscoveredCollection, KindleStorage, ProgressStore, SettingsStore, StoragePaths,
};
use chess_core::{
    puzzle_app::{PuzzleAction as Action, PuzzleEffect as Effect, PuzzleState},
    ActiveCollection, BoardSessionStatus, Settings, Workspace,
};
use chess_render::{HitTarget, ScreenState};

pub struct PuzzleRuntime {
    state: PuzzleState,
    storage: KindleStorage,
    progress: ProgressStore,
    settings: SettingsStore,
    preferences: Settings,
    persistent: bool,
    board: BoardService,
}
impl PuzzleRuntime {
    pub fn open(paths: StoragePaths) -> Result<Self, Box<dyn std::error::Error>> {
        let storage = KindleStorage::new(paths);
        let entries = storage.prepare_collections(BUNDLED_PUZZLES)?;
        let (mut progress, progress_load) = ProgressStore::open(&storage.paths().progress_file);
        let (settings, settings_load) = SettingsStore::open(&storage.paths().settings_file);
        let (key, collection, persistent, warning) =
            select_initial_collection(&storage, &entries, &progress_load.progress)?;
        let preferences = settings_load.settings;
        let mut state = PuzzleState::new_with_settings(
            ActiveCollection::from_collection(key, collection),
            progress_load.progress.clone(),
            preferences.for_workspace(Workspace::Puzzles).clone(),
            preferences.physical_board_seat(),
        );
        state.set_collection_entries(
            entries
                .iter()
                .map(DiscoveredCollection::as_core_entry)
                .collect(),
        );
        let warnings = [progress_load.warning, settings_load.warning, warning]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if !warnings.is_empty() {
            state.dispatch(Action::SetTransientMessage(Some(warnings.join(" "))));
        }
        if persistent && state.progress() != &progress_load.progress {
            progress.mark_dirty();
        }
        let mut runtime = Self {
            state,
            storage,
            progress,
            settings,
            preferences,
            persistent,
            board: BoardService::new(),
        };
        runtime.flush();
        Ok(runtime)
    }
    pub fn state(&self) -> &PuzzleState {
        &self.state
    }
    pub fn dispatch(&mut self, action: Action) -> bool {
        let mut queue = std::collections::VecDeque::from(self.state.dispatch(action));
        let mut exit = false;
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Board(intent) => {
                    if let Err(reason) = self.board.send(intent) {
                        self.state
                            .dispatch(Action::BoardStatus(BoardSessionStatus::Error(reason)));
                    }
                }
                Effect::ExitRequested => exit = true,
                Effect::ProgressChanged => {
                    if self.persistent {
                        self.progress.mark_dirty();
                    }
                }
                Effect::SettingsChanged => self.settings.mark_dirty(),
                Effect::CollectionRequested(key) => match self.storage.load_collection(&key) {
                    Ok(collection) => {
                        queue.extend(self.state.dispatch(Action::ActivateCollection(
                            ActiveCollection::from_collection(key, collection),
                        )))
                    }
                    Err(error) => {
                        self.state.dispatch(Action::CloseCollectionPicker);
                        self.state
                            .dispatch(Action::SetTransientMessage(Some(format!(
                                "Collection error: {error}"
                            ))));
                    }
                },
            }
        }
        self.flush();
        exit
    }
}
impl Application for PuzzleRuntime {
    fn screen(&self) -> &impl ScreenState {
        &self.state
    }
    fn poll(&mut self) -> Poll {
        let mut changed = false;
        loop {
            match self.board.poll() {
                Ok(Some(update)) => {
                    let action = match update {
                        BoardUpdate::Status(status) => Action::BoardStatus(status),
                        BoardUpdate::Position(position) => Action::BoardPosition(position),
                    };
                    if self.dispatch(action) {
                        return Poll::Exit;
                    }
                    changed = true;
                }
                Ok(None) => break,
                Err(reason) => {
                    let status = BoardSessionStatus::Error(reason);
                    if self.state.board_status() != &status {
                        self.state.dispatch(Action::BoardStatus(status));
                        changed = true;
                    }
                    break;
                }
            }
        }
        if changed {
            Poll::Changed
        } else {
            Poll::Idle
        }
    }
    fn tap(&mut self, target: HitTarget) -> bool {
        target
            .into_puzzle_action()
            .is_some_and(|action| self.dispatch(action))
    }
    fn pause_input(&mut self) {
        self.state.dispatch(Action::PauseBoardInput);
    }
    fn flush(&mut self) {
        self.preferences
            .set_workspace_preferences(Workspace::Puzzles, self.state.settings().clone());
        self.preferences
            .set_physical_board_seat(self.state.physical_board_seat());
        let mut warnings = Vec::new();
        if self.persistent {
            if let Err(error) = self.progress.retry_if_dirty(self.state.progress()) {
                warnings.push(format!("Progress warning: {error}"));
            }
        }
        if let Err(error) = self.settings.retry_if_dirty(&self.preferences) {
            warnings.push(format!("Settings warning: {error}"));
        }
        if !warnings.is_empty() {
            self.state
                .dispatch(Action::SetTransientMessage(Some(warnings.join(" "))));
        } else if self
            .state
            .transient_message()
            .is_some_and(super::is_persistence_warning)
        {
            self.state.dispatch(Action::SetTransientMessage(None));
        }
    }
}
