//! Durable authored review position, independent of puzzle progress and scratch boards.
use crate::{parse_fen, parse_uci_move, ReviewGame, ReviewState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewResume {
    version: u32,
    pub collection_id: String,
    pub game_id: String,
    pub move_path: Vec<String>,
    pub fen: String,
    pub flipped: bool,
    pub orientation_locked: bool,
    pub analysis_page: usize,
    pub focus_selected: bool,
}

impl ReviewResume {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let resume: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        if resume.version != 1 {
            return Err(format!(
                "Unsupported review resume version {}",
                resume.version
            ));
        }
        if resume.collection_id.trim().is_empty() || resume.game_id.trim().is_empty() {
            return Err("Review resume identities must be nonempty".into());
        }
        parse_fen(&resume.fen).map_err(|error| error.to_string())?;
        for movement in &resume.move_path {
            parse_uci_move(movement).map_err(|error| error.to_string())?;
        }
        Ok(resume)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self).map_err(|error| error.to_string())
    }

    pub(crate) fn capture(collection_id: &str, game: &ReviewGame, state: &ReviewState) -> Self {
        let mut index = state.selected_node();
        let selected = game
            .analysis
            .node(index)
            .expect("selected review node exists");
        let fen = selected.fen.clone();
        let mut move_path = Vec::new();
        while let Some(node) = game.analysis.node(index) {
            let Some(parent) = node.parent else { break };
            move_path.push(node.movement.as_ref().expect("non-root move").uci.clone());
            index = parent;
        }
        move_path.reverse();
        Self {
            version: 1,
            collection_id: collection_id.into(),
            game_id: game.id.clone(),
            move_path,
            fen,
            flipped: state.flipped(),
            orientation_locked: state.orientation_locked(),
            analysis_page: state.analysis_page(),
            focus_selected: state.analysis_focus().is_some(),
        }
    }

    pub(crate) fn selected_node(&self, game: &ReviewGame) -> Option<crate::AnalysisNodeIndex> {
        let mut index = game.analysis.root_index();
        for movement in &self.move_path {
            let node = game.analysis.node(index)?;
            let mut matches = node.children.iter().copied().filter(|child| {
                game.analysis
                    .node(*child)
                    .and_then(|node| node.movement.as_ref())
                    .is_some_and(|candidate| &candidate.uci == movement)
            });
            index = matches.next()?;
            if matches.next().is_some() {
                return None;
            }
        }
        (game.analysis.node(index)?.fen == self.fen).then_some(index)
    }
}
