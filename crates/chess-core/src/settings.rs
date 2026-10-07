//! Durable application preferences shared across platform frontends.

use serde::{Deserialize, Serialize};

pub const SETTINGS_VERSION: u32 = 1;

const fn enabled_by_default() -> bool {
    true
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    version: u32,
    #[serde(default = "enabled_by_default")]
    show_free_mode_button: bool,
    #[serde(default = "enabled_by_default")]
    show_notes_button: bool,
    #[serde(default)]
    small_board: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            show_free_mode_button: true,
            show_notes_button: true,
            small_board: false,
        }
    }
}

impl Settings {
    pub const fn version(&self) -> u32 {
        self.version
    }

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

    pub fn parse(contents: &[u8]) -> Result<Self, String> {
        let settings: Self = serde_json::from_slice(contents)
            .map_err(|error| format!("Invalid settings.v1: {error}"))?;
        if settings.version != SETTINGS_VERSION {
            return Err(format!(
                "Unsupported settings version {}; expected {}.",
                settings.version, SETTINGS_VERSION
            ));
        }
        Ok(settings)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|error| format!("Could not encode settings: {error}"))
    }
}
