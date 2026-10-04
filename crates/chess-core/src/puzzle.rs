//! Version-1 puzzle collection parsing.

use crate::analysis::{fallback_text, parse_analysis, parse_description_content};
use crate::{parse_fen, parse_uci_move, AnalysisTextSpan, AnalysisTree, Color};
use serde::{de::Error as _, Deserialize, Deserializer};
use std::{collections::HashSet, fmt};

pub const LEGACY_PUZZLE_FILE_WARNING_BYTES: usize = 256 * 1024;
pub const MAX_PUZZLE_FILE_BYTES: usize = 8 * 1024 * 1024;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuzzleCollection {
    version: u32,
    pub title: Option<String>,
    pub puzzles: Vec<Puzzle>,
}

impl PuzzleCollection {
    pub const fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    pub description: Option<String>,
    pub description_content: Vec<AnalysisTextSpan>,
    pub difficulty: Option<Difficulty>,
    pub source: Option<String>,
    /// Alternating solver and opponent moves, starting with the solver move.
    pub solution: Vec<String>,
    pub analysis: Option<AnalysisTree>,
}

impl Puzzle {
    pub fn side_to_move(&self) -> Color {
        match self.fen.split_ascii_whitespace().nth(1) {
            Some("b") => Color::Black,
            _ => Color::White,
        }
    }
}

#[derive(Deserialize)]
struct RawPuzzleCollection {
    version: u32,
    #[serde(default)]
    title: Option<String>,
    puzzles: Vec<RawPuzzle>,
}

#[derive(Deserialize)]
struct RawPuzzle {
    id: String,
    fen: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    description_content: Option<serde_json::Value>,
    #[serde(default)]
    difficulty: Option<Difficulty>,
    #[serde(default)]
    source: Option<String>,
    solution: Vec<String>,
    #[serde(default)]
    analysis: Option<serde_json::Value>,
}

impl<'de> Deserialize<'de> for PuzzleCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPuzzleCollection::deserialize(deserializer)?;
        build_collection(raw).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Puzzle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPuzzle::deserialize(deserializer)?;
        let id = raw.id.clone();
        build_puzzle(raw, &format!("Puzzle ({id})")).map_err(D::Error::custom)
    }
}

pub fn parse_puzzle_file(contents: &[u8]) -> Result<PuzzleCollection, String> {
    if contents.len() > MAX_PUZZLE_FILE_BYTES {
        return Err("Puzzle file must be at most 8 MiB (8,388,608 bytes).".into());
    }

    let raw: RawPuzzleCollection = serde_json::from_slice(contents)
        .map_err(|error| format!("Invalid puzzle file: {error}"))?;
    build_collection(raw)
}

fn build_collection(mut raw: RawPuzzleCollection) -> Result<PuzzleCollection, String> {
    if raw.version != 1 {
        return Err(format!(
            "Unsupported puzzle file version {}; expected 1.",
            raw.version
        ));
    }
    if raw.puzzles.is_empty() {
        return Err("No puzzles found. Add at least one puzzle to the file.".into());
    }

    raw.title = raw
        .title
        .take()
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty());

    let mut ids = HashSet::new();
    let mut puzzles = Vec::with_capacity(raw.puzzles.len());
    for (index, raw_puzzle) in raw.puzzles.into_iter().enumerate() {
        let name = format!("Puzzle {} ({})", index + 1, raw_puzzle.id);
        if raw_puzzle.id.trim().is_empty() || !ids.insert(raw_puzzle.id.clone()) {
            return Err(format!("{name}: id must be nonempty and unique."));
        }
        puzzles.push(build_puzzle(raw_puzzle, &name)?);
    }

    Ok(PuzzleCollection {
        version: raw.version,
        title: raw.title,
        puzzles,
    })
}

fn build_puzzle(raw: RawPuzzle, name: &str) -> Result<Puzzle, String> {
    parse_fen(&raw.fen).map_err(|error| format!("{name}: {error}"))?;
    if raw.solution.is_empty() {
        return Err(format!(
            "{name}: solution must contain at least one UCI move."
        ));
    }
    for (move_index, movement) in raw.solution.iter().enumerate() {
        if parse_uci_move(movement).is_err() {
            return Err(format!(
                "{name}: solution move {} must use UCI notation (for example e2e4 or a7a8q).",
                move_index + 1
            ));
        }
    }

    let analysis = raw
        .analysis
        .as_ref()
        .map(|value| parse_analysis(value, &raw.fen, &raw.solution, name))
        .transpose()?;

    let description_content = match raw.description_content.as_ref() {
        Some(value) => parse_description_content(
            value,
            analysis.as_ref(),
            &format!("{name}: description_content"),
        )?,
        None => fallback_text(raw.description.as_deref()),
    };

    Ok(Puzzle {
        id: raw.id,
        fen: raw.fen,
        description: raw.description,
        description_content,
        difficulty: raw.difficulty,
        source: raw.source,
        solution: raw.solution,
        analysis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnalysisRole, AnalysisTextSpan};

    const VALID_RICH: &[u8] =
        include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
    const LEGACY: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
    const INVALID_RICH: &[(&str, &[u8])] = &[
        (
            "duplicate id",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-duplicate-id.json"),
        ),
        (
            "missing child",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-missing-child.json"),
        ),
        (
            "cycle",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-cycle.json"),
        ),
        (
            "disconnected node",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-disconnected.json"),
        ),
        (
            "root FEN",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-root-fen-mismatch.json"),
        ),
        (
            "invalid UCI",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-uci.json"),
        ),
        (
            "main solution",
            include_bytes!(
                "../../../tests/fixtures/rich-analysis/invalid-main-solution-mismatch.json"
            ),
        ),
        (
            "dangling move ref",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-move-ref-dangling.json"),
        ),
        (
            "root move ref",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-move-ref-root.json"),
        ),
        (
            "missing text",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-span-missing-text.json"),
        ),
        (
            "unknown span",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-span-unknown-type.json"),
        ),
        (
            "bad label",
            include_bytes!("../../../tests/fixtures/rich-analysis/invalid-span-bad-label.json"),
        ),
    ];

    #[test]
    fn parses_valid_rich_fixture_into_indexed_model() {
        let collection = parse_puzzle_file(VALID_RICH).expect("valid rich fixture");
        let puzzle = &collection.puzzles[0];
        let analysis = puzzle.analysis.as_ref().expect("analysis tree");

        assert_eq!(analysis.version(), 1);
        assert_eq!(analysis.nodes().len(), 4);
        assert_eq!(analysis.root().id, "n0");
        assert_eq!(
            analysis.root().position,
            parse_fen(&puzzle.fen).expect("root FEN")
        );

        let n1_index = analysis.node_index("n1").expect("n1");
        let n1 = analysis.node(n1_index).expect("n1 by index");
        assert_eq!(n1.parent, Some(analysis.root_index()));
        assert_eq!(n1.role, Some(AnalysisRole::Main));
        let movement = n1.movement.as_ref().expect("n1 move");
        assert_eq!(movement.uci, "h2h1q");
        assert_eq!(
            movement.parsed_uci,
            parse_uci_move("h2h1q").expect("promotion UCI")
        );
        assert_eq!(movement.san, "h1=Q+");
        assert_eq!(n1.nags[0].value(), 1);

        let n2 = analysis.node_by_id("n2").expect("n2");
        assert_eq!(n2.role, Some(AnalysisRole::Alternative));
        assert_eq!(
            n2.content,
            vec![AnalysisTextSpan::Text(
                "An explicitly marked alternative.".to_owned()
            )]
        );

        let n3 = analysis.node_by_id("n3").expect("n3");
        assert_eq!(n3.role, Some(AnalysisRole::Sideline));
        assert!(n3.comment.is_empty());
        assert!(n3.content.is_empty());
        assert!(n3.nags.is_empty());
        assert!(n3.children.is_empty());

        match &puzzle.description_content[1] {
            AnalysisTextSpan::MoveRef { node, label } => {
                assert_eq!(analysis.node(*node).expect("description target").id, "n1");
                assert_eq!(label.as_deref(), Some("1...h1=Q+"));
            }
            AnalysisTextSpan::Text(_) => panic!("expected move_ref span"),
        }
        match &puzzle.description_content[3] {
            AnalysisTextSpan::MoveRef { node, label } => {
                assert_eq!(analysis.node(*node).expect("description target").id, "n2");
                assert_eq!(label, &None);
            }
            AnalysisTextSpan::Text(_) => panic!("expected move_ref span"),
        }
    }

    #[test]
    fn rejects_every_task_20_invalid_fixture_with_puzzle_context() {
        for (label, fixture) in INVALID_RICH {
            let error = parse_puzzle_file(fixture).expect_err(label);
            assert!(
                error.contains("Puzzle 1 (rich-black-promotion)"),
                "{label}: {error}"
            );
        }
    }

    #[test]
    fn preserves_legacy_fixture_public_semantics() {
        let collection = parse_puzzle_file(LEGACY).expect("legacy fixture");
        assert_eq!(collection.version(), 1);
        assert_eq!(collection.title.as_deref(), Some("Lichess sample puzzles"));
        assert_eq!(collection.puzzles.len(), 2);

        let first = &collection.puzzles[0];
        assert_eq!(first.id, "lichess-001cr");
        assert_eq!(first.fen, "8/3B2pp/p5k1/6P1/1ppp1K2/8/1P6/8 w - - 0 39");
        assert_eq!(first.solution, vec!["d7e8".to_owned()]);
        assert_eq!(
            first.source.as_deref(),
            Some("https://lichess.org/training/001cr")
        );
        assert!(first.analysis.is_none());
        assert_eq!(
            first.description_content,
            vec![AnalysisTextSpan::Text(
                first.description.clone().expect("legacy description")
            )]
        );
    }

    #[test]
    fn phase_two_cap_accepts_files_above_legacy_warning_threshold() {
        let padding = "x".repeat(LEGACY_PUZZLE_FILE_WARNING_BYTES);
        let document = format!(
            r#"{{"version":1,"padding":"{padding}","puzzles":[{{"id":"large","fen":"8/8/8/8/8/8/8/K6k w - - 0 1","solution":["a1a2"]}}]}}"#
        );
        assert!(document.len() > LEGACY_PUZZLE_FILE_WARNING_BYTES);
        assert!(document.len() < MAX_PUZZLE_FILE_BYTES);
        parse_puzzle_file(document.as_bytes()).expect("phase-two-sized file");
    }

    #[test]
    fn phase_two_cap_rejects_files_above_eight_mib() {
        let document = vec![b' '; MAX_PUZZLE_FILE_BYTES + 1];
        assert_eq!(
            parse_puzzle_file(&document).expect_err("oversized file"),
            "Puzzle file must be at most 8 MiB (8,388,608 bytes)."
        );
    }

    #[test]
    fn rejects_additional_frozen_contract_invariants() {
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["version"] = serde_json::json!(2);
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][0]["parent"] = serde_json::Value::Null;
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]["role"] = serde_json::json!("other");
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]["nags"] = serde_json::json!([-1]);
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][0]["children"] =
                serde_json::json!(["n1", "n1", "n2", "n3"]);
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]["move"]["extra"] = serde_json::json!(true);
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]["move"]["san"] = serde_json::json!("");
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]["fen"] = serde_json::json!("not a FEN");
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][1]
                .as_object_mut()
                .expect("node object")
                .remove("parent");
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["description_content"][0]["extra"] = serde_json::json!(true);
        });
        assert_invalid_mutation(|puzzle| {
            puzzle["analysis"]["nodes"][2]["children"] = serde_json::json!(["n3"]);
        });
    }

    fn assert_invalid_mutation(mutate: impl FnOnce(&mut serde_json::Value)) {
        let mut document: serde_json::Value =
            serde_json::from_slice(VALID_RICH).expect("valid fixture JSON");
        let puzzle = &mut document["puzzles"][0];
        mutate(puzzle);
        let encoded = serde_json::to_vec(&document).expect("mutated fixture JSON");
        assert!(parse_puzzle_file(&encoded).is_err());
    }
}
