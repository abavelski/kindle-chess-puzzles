//! Version-1 game-review collection parsing and platform-neutral model.

use crate::analysis::parse_review_analysis;
use crate::{parse_fen, AnalysisTree};
use serde::{de::Error as _, Deserialize, Deserializer};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

pub const MAX_REVIEW_FILE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewCollection {
    version: u32,
    pub title: Option<String>,
    pub source: Option<String>,
    pub games: Vec<ReviewGame>,
}

impl ReviewCollection {
    pub const fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewGame {
    pub id: String,
    pub fen: String,
    pub metadata: ReviewMetadata,
    pub analysis: AnalysisTree,
}

impl ReviewGame {
    pub fn key(&self, collection_id: impl Into<String>) -> ReviewGameKey {
        ReviewGameKey::new(collection_id, self.id.clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewMetadata {
    pub white: String,
    pub black: String,
    pub result: ReviewResult,
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub source: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewResult {
    WhiteWin,
    BlackWin,
    Draw,
    Ongoing,
}

impl ReviewResult {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WhiteWin => "1-0",
            Self::BlackWin => "0-1",
            Self::Draw => "1/2-1/2",
            Self::Ongoing => "*",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "1-0" => Some(Self::WhiteWin),
            "0-1" => Some(Self::BlackWin),
            "1/2-1/2" => Some(Self::Draw),
            "*" => Some(Self::Ongoing),
            _ => None,
        }
    }
}

impl fmt::Display for ReviewResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable platform-neutral identity for one review game in a flattened multi-collection picker.
///
/// The collection identifier is intentionally just an opaque string. A platform layer may use a
/// stable filename such as `games-classics.json`, but core never interprets it as a filesystem path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReviewGameKey {
    collection_id: String,
    game_id: String,
}

impl ReviewGameKey {
    pub fn new(collection_id: impl Into<String>, game_id: impl Into<String>) -> Self {
        Self {
            collection_id: collection_id.into(),
            game_id: game_id.into(),
        }
    }

    pub fn collection_id(&self) -> &str {
        &self.collection_id
    }

    pub fn game_id(&self) -> &str {
        &self.game_id
    }
}

#[derive(Deserialize)]
struct RawReviewCollection {
    version: u32,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    source: Option<String>,
    games: Vec<RawReviewGame>,
}

#[derive(Deserialize)]
struct RawReviewGame {
    id: String,
    fen: String,
    white: String,
    black: String,
    result: String,
    event: String,
    site: String,
    date: String,
    round: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    source: Option<String>,
    analysis: Value,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(Some(value)),
        _ => Err(D::Error::custom("value must be a string when present")),
    }
}

pub fn parse_review_file(contents: &[u8]) -> Result<ReviewCollection, String> {
    if contents.len() > MAX_REVIEW_FILE_BYTES {
        return Err("Review file must be at most 8 MiB (8,388,608 bytes).".into());
    }

    let raw: RawReviewCollection = serde_json::from_slice(contents)
        .map_err(|error| format!("Invalid review file: {error}"))?;
    build_collection(raw)
}

fn build_collection(raw: RawReviewCollection) -> Result<ReviewCollection, String> {
    if raw.version != 1 {
        return Err(format!(
            "Unsupported review file version {}; expected 1.",
            raw.version
        ));
    }
    if raw.games.is_empty() {
        return Err("No review games found. Add at least one game to the file.".into());
    }

    let mut ids = HashSet::new();
    let mut games = Vec::with_capacity(raw.games.len());
    for (index, raw_game) in raw.games.into_iter().enumerate() {
        let name = format!("Game {} ({})", index + 1, raw_game.id);
        if raw_game.id.trim().is_empty() || !ids.insert(raw_game.id.clone()) {
            return Err(format!("{name}: id must be nonempty and unique."));
        }
        games.push(build_game(raw_game, &name)?);
    }

    Ok(ReviewCollection {
        version: raw.version,
        title: raw.title,
        source: raw.source,
        games,
    })
}

fn build_game(raw: RawReviewGame, name: &str) -> Result<ReviewGame, String> {
    if raw.extra.contains_key("solution") {
        return Err(format!(
            "{name}: review games must not contain a solution field."
        ));
    }

    parse_fen(&raw.fen).map_err(|error| format!("{name}: {error}"))?;

    for (field, value) in [
        ("white", raw.white.as_str()),
        ("black", raw.black.as_str()),
        ("event", raw.event.as_str()),
        ("site", raw.site.as_str()),
        ("date", raw.date.as_str()),
        ("round", raw.round.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name}: {field} must be a non-empty string."));
        }
    }

    let result = ReviewResult::parse(&raw.result)
        .ok_or_else(|| format!("{name}: result must be one of 1-0, 0-1, 1/2-1/2, or *."))?;
    let analysis = parse_review_analysis(&raw.analysis, &raw.fen, name)?;

    Ok(ReviewGame {
        id: raw.id,
        fen: raw.fen,
        metadata: ReviewMetadata {
            white: raw.white,
            black: raw.black,
            result,
            event: raw.event,
            site: raw.site,
            date: raw.date,
            round: raw.round,
            source: raw.source,
        },
        analysis,
    })
}
