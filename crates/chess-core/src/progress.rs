//! Durable learning progress independent from puzzle collection source data.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROGRESS_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    version: u32,
    #[serde(default)]
    pub active_file: Option<String>,
    #[serde(default)]
    pub files: BTreeMap<String, FileProgress>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileProgress {
    #[serde(default)]
    pub current_puzzle_id: Option<String>,
    #[serde(default)]
    pub solved_ids: BTreeSet<String>,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress {
    pub fn new() -> Self {
        Self {
            version: PROGRESS_VERSION,
            active_file: None,
            files: BTreeMap::new(),
        }
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub fn parse(contents: &[u8]) -> Result<Self, String> {
        let progress: Self = serde_json::from_slice(contents)
            .map_err(|error| format!("Invalid progress.v1: {error}"))?;
        if progress.version != PROGRESS_VERSION {
            return Err(format!(
                "Unsupported progress version {}; expected {}.",
                progress.version, PROGRESS_VERSION
            ));
        }
        Ok(progress)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|error| format!("Could not encode progress: {error}"))
    }

    pub fn file(&self, key: &str) -> Option<&FileProgress> {
        self.files.get(key)
    }

    pub fn remember_puzzle(&mut self, key: &str, puzzle_id: &str) -> bool {
        let file = self.files.entry(key.to_owned()).or_default();
        if file.current_puzzle_id.as_deref() == Some(puzzle_id) {
            false
        } else {
            file.current_puzzle_id = Some(puzzle_id.to_owned());
            true
        }
    }

    pub fn set_active_file(&mut self, key: &str) -> bool {
        if self.active_file.as_deref() == Some(key) {
            false
        } else {
            self.active_file = Some(key.to_owned());
            true
        }
    }

    pub fn mark_solved(&mut self, key: &str, puzzle_id: &str) -> bool {
        self.files
            .entry(key.to_owned())
            .or_default()
            .solved_ids
            .insert(puzzle_id.to_owned())
    }

    pub fn is_solved(&self, key: &str, puzzle_id: &str) -> bool {
        self.files
            .get(key)
            .is_some_and(|file| file.solved_ids.contains(puzzle_id))
    }
}
