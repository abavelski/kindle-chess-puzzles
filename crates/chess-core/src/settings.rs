//! Durable application preferences shared across platform frontends.

use crate::Workspace;
use serde::{Deserialize, Serialize};

pub const SETTINGS_VERSION: u32 = 1;

const fn enabled_by_default() -> bool {
    true
}

/// Preferences for one workspace, independent of the other workspace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    #[serde(default = "enabled_by_default")]
    show_free_mode_button: bool,
    #[serde(default = "enabled_by_default")]
    show_notes_button: bool,
    #[serde(default)]
    small_board: bool,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            show_free_mode_button: true,
            show_notes_button: true,
            small_board: false,
        }
    }
}

impl WorkspaceSettings {
    pub const fn show_free_mode_button(&self) -> bool {
        self.show_free_mode_button
    }

    pub const fn show_notes_button(&self) -> bool {
        self.show_notes_button
    }

    pub fn set_show_free_mode_button(&mut self, value: bool) {
        self.show_free_mode_button = value;
    }

    pub fn set_show_notes_button(&mut self, value: bool) {
        self.show_notes_button = value;
    }

    pub const fn small_board(&self) -> bool {
        self.small_board
    }

    pub fn set_small_board(&mut self, value: bool) {
        self.small_board = value;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    version: u32,
    // Retain the version-one flat fields as the puzzle preferences.
    #[serde(flatten)]
    puzzles: WorkspaceSettings,
    #[serde(default)]
    game_review: Option<WorkspaceSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            puzzles: WorkspaceSettings::default(),
            game_review: Some(WorkspaceSettings::default()),
        }
    }
}

impl Settings {
    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn for_workspace(&self, workspace: Workspace) -> &WorkspaceSettings {
        match workspace {
            Workspace::Puzzles => &self.puzzles,
            Workspace::Review => match &self.game_review {
                Some(settings) => settings,
                None => &self.puzzles,
            },
        }
    }

    pub(crate) fn for_workspace_mut(&mut self, workspace: Workspace) -> &mut WorkspaceSettings {
        match workspace {
            Workspace::Puzzles => &mut self.puzzles,
            Workspace::Review => self.game_review.get_or_insert_with(|| self.puzzles.clone()),
        }
    }

    pub const fn show_free_mode_button(&self) -> bool {
        self.puzzles.show_free_mode_button()
    }

    pub const fn show_notes_button(&self) -> bool {
        self.puzzles.show_notes_button()
    }

    pub fn set_show_free_mode_button(&mut self, value: bool) {
        self.puzzles.set_show_free_mode_button(value);
    }

    pub fn set_show_notes_button(&mut self, value: bool) {
        self.puzzles.set_show_notes_button(value);
    }

    pub const fn small_board(&self) -> bool {
        self.puzzles.small_board()
    }

    pub fn set_small_board(&mut self, value: bool) {
        self.puzzles.set_small_board(value);
    }

    pub fn parse(contents: &[u8]) -> Result<Self, String> {
        let mut settings: Self = serde_json::from_slice(contents)
            .map_err(|error| format!("Invalid settings.v1: {error}"))?;
        if settings.version != SETTINGS_VERSION {
            return Err(format!(
                "Unsupported settings version {}; expected {}.",
                settings.version, SETTINGS_VERSION
            ));
        }
        // Freeze the old global preference values into independent profiles.
        if settings.game_review.is_none() {
            settings.game_review = Some(settings.puzzles.clone());
        }
        Ok(settings)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|error| format!("Could not encode settings: {error}"))
    }
}
