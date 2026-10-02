//! Version-1 puzzle collection parsing.

use crate::{parse_fen, parse_uci_move, Color};
use serde::Deserialize;
use std::{collections::HashSet, fmt};

pub const MAX_PUZZLE_FILE_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Difficulty {
    Text(String),
    Number(serde_json::Number),
}

impl fmt::Display for Difficulty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => formatter.write_str(text),
            Self::Number(number) => write!(formatter, "{number}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct PuzzleCollection {
    version: u32,
    #[serde(default)]
    pub title: Option<String>,
    pub puzzles: Vec<Puzzle>,
}

impl PuzzleCollection {
    pub const fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub difficulty: Option<Difficulty>,
    #[serde(default)]
    pub source: Option<String>,
    /// Alternating solver and opponent moves, starting with the solver move.
    pub solution: Vec<String>,
}

impl Puzzle {
    pub fn side_to_move(&self) -> Color {
        match self.fen.split_ascii_whitespace().nth(1) {
            Some("b") => Color::Black,
            _ => Color::White,
        }
    }
}

pub fn parse_puzzle_file(contents: &[u8]) -> Result<PuzzleCollection, String> {
    if contents.len() > MAX_PUZZLE_FILE_BYTES {
        return Err("Puzzle file must be at most 256 KiB.".into());
    }

    let mut file: PuzzleCollection = serde_json::from_slice(contents)
        .map_err(|error| format!("Invalid puzzle file: {error}"))?;
    if file.version != 1 {
        return Err(format!(
            "Unsupported puzzle file version {}; expected 1.",
            file.version
        ));
    }
    if file.puzzles.is_empty() {
        return Err("No puzzles found. Add at least one puzzle to the file.".into());
    }

    file.title = file
        .title
        .take()
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty());

    let mut ids = HashSet::new();
    for (index, puzzle) in file.puzzles.iter().enumerate() {
        let name = format!("Puzzle {} ({})", index + 1, puzzle.id);
        if puzzle.id.trim().is_empty() || !ids.insert(puzzle.id.as_str()) {
            return Err(format!("{name}: id must be nonempty and unique."));
        }
        parse_fen(&puzzle.fen).map_err(|error| format!("{name}: {error}"))?;
        if puzzle.solution.is_empty() {
            return Err(format!(
                "{name}: solution must contain at least one UCI move."
            ));
        }
        for (move_index, movement) in puzzle.solution.iter().enumerate() {
            if parse_uci_move(movement).is_err() {
                return Err(format!(
                    "{name}: solution move {} must use UCI notation (for example e2e4 or a7a8q).",
                    move_index + 1
                ));
            }
        }
    }

    Ok(file)
}
