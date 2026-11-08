use crate::{runtime::Application, KindleStorage, ReviewResumeStore, SettingsStore, StoragePaths};
use chess_core::{
    review_app::{ReviewAction as Action, ReviewApp, ReviewEffect as Effect},
    Settings, Workspace,
};
use chess_render::{HitTarget, ScreenState};
use std::path::Path;

pub fn review_paths(root: &Path) -> StoragePaths {
    let mut paths = StoragePaths::new(root.join("puzzles"), root.join("state/progress.json"));
    paths.review_dir = root.join("games");
    paths
}
/// Games-only composition: no puzzle state, progress store or board service.
pub struct ReviewRuntime {
    state: ReviewApp,
    storage: KindleStorage,
    settings: SettingsStore,
    preferences: Settings,
    resume: ReviewResumeStore,
}
impl ReviewRuntime {
    pub fn open(paths: StoragePaths) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_with_legacy(paths, None)
    }
    pub fn open_with_legacy(
        mut paths: StoragePaths,
        legacy: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut migration_warnings = Vec::new();
        let mut library = KindleStorage::new(paths.clone()).discover_review_library()?;
        if library.games.is_empty() && library.errors.is_empty() {
            if let Some(root) = legacy {
                paths.review_dir = root.join("games");
                library = KindleStorage::new(paths.clone()).discover_review_library()?;
            }
        }
        let storage = KindleStorage::new(paths);
        let (settings, mut settings_load) = SettingsStore::open(&storage.paths().settings_file);
        let (resume, mut resume_load) =
            ReviewResumeStore::open(&storage.paths().review_resume_file);
        if let Some(root) = legacy {
            if missing(&storage.paths().settings_file) {
                if let Some(value) = read_legacy(
                    &root.join("state/settings.json"),
                    Settings::parse,
                    &mut migration_warnings,
                ) {
                    settings_load.settings = value;
                }
            }
            if missing(&storage.paths().review_resume_file) {
                if let Some(value) = read_legacy(
                    &root.join("state/review-resume.json"),
                    chess_core::ReviewResume::parse,
                    &mut migration_warnings,
                ) {
                    resume_load.resume = Some(value);
                }
            }
        }
        let preferences = settings_load.settings;
        let mut state = ReviewApp::new(
            library.games,
            preferences.for_workspace(Workspace::Review).clone(),
        );
        state.set_review_file_errors(library.errors);
        let mut warnings = [settings_load.warning, resume_load.warning]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        warnings.extend(migration_warnings);
        if let Some(saved) = resume_load.resume {
            if let Some(warning) = state.restore_review_resume(&saved) {
                warnings.push(warning);
            }
        }
        if !warnings.is_empty() {
            state.dispatch(Action::SetTransientMessage(Some(warnings.join(" "))));
        }
        Ok(Self {
            state,
            storage,
            settings,
            preferences,
            resume,
        })
    }
    pub fn state(&self) -> &ReviewApp {
        &self.state
    }
    pub fn dispatch(&mut self, action: Action) -> bool {
        let mut queue = std::collections::VecDeque::from(self.state.dispatch(action));
        let mut exit = false;
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::ExitRequested => exit = true,
                Effect::SettingsChanged => self.settings.mark_dirty(),
                Effect::ReviewGameRequested(key) => match self.storage.load_review_game(&key) {
                    Ok(game) => queue.extend(
                        self.state
                            .dispatch(Action::ActivateReviewGame(key, Box::new(game))),
                    ),
                    Err(error) => {
                        self.state.dispatch(Action::CloseReviewGamePicker);
                        self.state
                            .dispatch(Action::SetTransientMessage(Some(format!(
                                "Review game error: {error}"
                            ))));
                    }
                },
            }
        }
        self.flush();
        exit
    }
}
impl Application for ReviewRuntime {
    fn screen(&self) -> &impl ScreenState {
        &self.state
    }
    fn tap(&mut self, target: HitTarget) -> bool {
        target
            .into_review_action()
            .is_some_and(|action| self.dispatch(action))
    }
    fn flush(&mut self) {
        self.preferences
            .set_workspace_preferences(Workspace::Review, self.state.settings().clone());
        let mut warnings = Vec::new();
        if let Err(error) = self.settings.retry_if_dirty(&self.preferences) {
            warnings.push(format!("Settings warning: {error}"));
        }
        if let Some(saved) = self.state.review_resume() {
            self.resume.observe(saved);
        }
        if let Err(error) = self.resume.retry_if_dirty() {
            warnings.push(format!("Review resume warning: {error}"));
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

fn missing(path: &Path) -> bool {
    matches!(path.symlink_metadata(), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
}
fn read_legacy<T>(
    path: &Path,
    parse: impl FnOnce(&[u8]) -> Result<T, String>,
    warnings: &mut Vec<String>,
) -> Option<T> {
    match std::fs::read(path) {
        Ok(bytes) => match parse(&bytes) {
            Ok(value) => Some(value),
            Err(error) => {
                warnings.push(format!("Legacy {} preserved: {error}", path.display()));
                None
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            warnings.push(format!("Legacy {} unavailable: {error}", path.display()));
            None
        }
    }
}
