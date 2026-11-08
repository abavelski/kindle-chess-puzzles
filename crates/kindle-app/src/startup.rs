use crate::{DiscoveredCollection, KindleStorage};
use chess_core::{parse_puzzle_file, Progress, PuzzleCollection};
pub const BUNDLED_PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/parity-puzzles.json");
const BUNDLED_FALLBACK_KEY: &str = "bundled-examples.json";
pub type InitialCollection = (String, PuzzleCollection, bool, Option<String>);
pub fn select_initial_collection(
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
