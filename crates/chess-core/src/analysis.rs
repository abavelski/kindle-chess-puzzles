//! Validated platform-neutral rich-analysis model.

use crate::fen::{parse_fen, FenPosition};
use crate::uci::{parse_uci_move, UciMove};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnalysisNodeIndex(usize);

impl AnalysisNodeIndex {
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRole {
    Main,
    Alternative,
    Sideline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Nag(u64);

impl Nag {
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisMove {
    pub uci: String,
    pub parsed_uci: UciMove,
    pub san: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisTextSpan {
    Text(String),
    MoveRef {
        node: AnalysisNodeIndex,
        label: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisNode {
    pub id: String,
    pub parent: Option<AnalysisNodeIndex>,
    pub movement: Option<AnalysisMove>,
    pub fen: String,
    pub position: FenPosition,
    pub role: Option<AnalysisRole>,
    pub comment: String,
    pub content: Vec<AnalysisTextSpan>,
    pub nags: Vec<Nag>,
    pub children: Vec<AnalysisNodeIndex>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisTree {
    version: u32,
    root: AnalysisNodeIndex,
    nodes: Vec<AnalysisNode>,
    index_by_id: HashMap<String, AnalysisNodeIndex>,
    main_line: Vec<AnalysisNodeIndex>,
    main_line_ply_by_node: Vec<Option<usize>>,
    nearest_main_line_ancestor_by_node: Vec<AnalysisNodeIndex>,
}

impl AnalysisTree {
    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn root_index(&self) -> AnalysisNodeIndex {
        self.root
    }

    pub fn root(&self) -> &AnalysisNode {
        &self.nodes[self.root.0]
    }

    pub fn nodes(&self) -> &[AnalysisNode] {
        &self.nodes
    }

    pub fn node(&self, index: AnalysisNodeIndex) -> Option<&AnalysisNode> {
        self.nodes.get(index.0)
    }

    pub fn node_index(&self, id: &str) -> Option<AnalysisNodeIndex> {
        self.index_by_id.get(id).copied()
    }

    pub fn node_by_id(&self, id: &str) -> Option<&AnalysisNode> {
        self.node_index(id).and_then(|index| self.node(index))
    }

    pub fn parent(&self, index: AnalysisNodeIndex) -> Option<&AnalysisNode> {
        self.node(index)
            .and_then(|node| node.parent)
            .and_then(|parent| self.node(parent))
    }

    pub fn children(&self, index: AnalysisNodeIndex) -> Option<&[AnalysisNodeIndex]> {
        self.node(index).map(|node| node.children.as_slice())
    }

    /// Root-inclusive ordered main line. Ply 0 is always the analysis root.
    pub fn main_line_nodes(&self) -> &[AnalysisNodeIndex] {
        &self.main_line
    }

    /// Returns the root at ply 0, the first authored main move at ply 1, and so on.
    pub fn main_line_node_at_ply(&self, ply: usize) -> Option<AnalysisNodeIndex> {
        self.main_line.get(ply).copied()
    }

    /// Returns a root-inclusive ply for main-line nodes, or None for variations.
    pub fn main_line_ply(&self, index: AnalysisNodeIndex) -> Option<usize> {
        self.main_line_ply_by_node.get(index.0).copied().flatten()
    }

    pub fn is_main_line(&self, index: AnalysisNodeIndex) -> bool {
        self.main_line_ply(index).is_some()
    }

    /// Returns the node itself when it is on the main line, otherwise the closest
    /// ancestor on the main line (the variation's branch point).
    pub fn nearest_main_line_ancestor(
        &self,
        index: AnalysisNodeIndex,
    ) -> Option<AnalysisNodeIndex> {
        self.nearest_main_line_ancestor_by_node.get(index.0).copied()
    }
}

#[derive(Clone, Debug)]
enum PendingTextSpan {
    Text(String),
    MoveRef { node: String, label: Option<String> },
}

#[derive(Clone, Debug)]
struct PendingNode {
    id: String,
    parent: Option<String>,
    movement: Option<AnalysisMove>,
    fen: String,
    position: FenPosition,
    role: Option<AnalysisRole>,
    comment: String,
    content: Vec<PendingTextSpan>,
    nags: Vec<Nag>,
    children: Vec<String>,
}

pub(crate) fn fallback_text(value: Option<&str>) -> Vec<AnalysisTextSpan> {
    value
        .filter(|text| !text.is_empty())
        .map(|text| vec![AnalysisTextSpan::Text(text.to_owned())])
        .unwrap_or_default()
}

pub(crate) fn parse_description_content(
    value: &Value,
    analysis: Option<&AnalysisTree>,
    context: &str,
) -> Result<Vec<AnalysisTextSpan>, String> {
    let pending = parse_pending_spans(value, context)?;
    resolve_spans_with_tree(pending, analysis, context)
}

pub(crate) fn parse_analysis(
    value: &Value,
    puzzle_fen: &str,
    solution: &[String],
    context: &str,
) -> Result<AnalysisTree, String> {
    let tree = parse_analysis_tree(value, puzzle_fen, "puzzle fen", context)?;
    validate_puzzle_main_path(&tree, solution, context)?;
    Ok(tree)
}

pub(crate) fn parse_review_analysis(
    value: &Value,
    game_fen: &str,
    context: &str,
) -> Result<AnalysisTree, String> {
    let tree = parse_analysis_tree(value, game_fen, "game fen", context)?;
    validate_review_main_path(&tree, context)?;
    Ok(tree)
}

fn parse_analysis_tree(
    value: &Value,
    expected_root_fen: &str,
    root_fen_label: &str,
    context: &str,
) -> Result<AnalysisTree, String> {
    let object = value
        .as_object()
        .ok_or_else(|| error(context, "analysis must be an object."))?;
    let version = required_u64(object, "version", context, "analysis")?;
    if version != 1 {
        return Err(error(
            context,
            &format!("unsupported analysis version {version}; expected 1."),
        ));
    }

    let root_id = required_nonempty_string(object, "root", context, "analysis")?;
    let raw_nodes = object
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| error(context, "analysis.nodes must be an array."))?;
    if raw_nodes.is_empty() {
        return Err(error(context, "analysis.nodes must not be empty."));
    }

    let mut node_objects = Vec::with_capacity(raw_nodes.len());
    let mut index_by_id = HashMap::with_capacity(raw_nodes.len());
    for (index, raw_node) in raw_nodes.iter().enumerate() {
        let node_context = format!("analysis node {}", index + 1);
        let node = raw_node
            .as_object()
            .ok_or_else(|| error(context, &format!("{node_context} must be an object.")))?;
        let id = required_nonempty_string(node, "id", context, &node_context)?;
        let node_index = AnalysisNodeIndex(index);
        if index_by_id.insert(id.clone(), node_index).is_some() {
            return Err(error(
                context,
                &format!("analysis node id {id:?} must be unique."),
            ));
        }
        node_objects.push((id, node));
    }

    let root = index_by_id.get(&root_id).copied().ok_or_else(|| {
        error(
            context,
            &format!("analysis.root {root_id:?} does not name an existing node."),
        )
    })?;

    let mut pending_nodes = Vec::with_capacity(node_objects.len());
    for (index, (id, node)) in node_objects.iter().enumerate() {
        let node_index = AnalysisNodeIndex(index);
        let is_root = node_index == root;
        let node_context = format!("analysis node {id:?}");

        let fen = required_string(node, "fen", context, &node_context)?;
        let position = parse_fen(&fen).map_err(|fen_error| {
            error(
                context,
                &format!("{node_context}: invalid FEN: {fen_error}."),
            )
        })?;
        if is_root && fen != expected_root_fen {
            return Err(error(
                context,
                &format!("{node_context}: root FEN must exactly match {root_fen_label}."),
            ));
        }

        let comment = optional_string(node, "comment", context, &node_context)?.unwrap_or_default();
        let content = match node.get("content") {
            Some(content) => parse_pending_spans(content, &format!("{node_context} content"))
                .map_err(|message| error(context, &message))?,
            None if comment.is_empty() => Vec::new(),
            None => vec![PendingTextSpan::Text(comment.clone())],
        };
        let children = optional_string_array(node, "children", context, &node_context)?;
        let mut unique_children = HashSet::with_capacity(children.len());
        for child in &children {
            if !unique_children.insert(child.as_str()) {
                return Err(error(
                    context,
                    &format!("{node_context}: child {child:?} is listed more than once."),
                ));
            }
        }

        let (parent, movement, role, nags) = if is_root {
            for forbidden in ["parent", "move", "role", "nags"] {
                if node.contains_key(forbidden) {
                    return Err(error(
                        context,
                        &format!("{node_context}: root node must not contain {forbidden:?}."),
                    ));
                }
            }
            (None, None, None, Vec::new())
        } else {
            let parent = Some(required_nonempty_string(
                node,
                "parent",
                context,
                &node_context,
            )?);
            let movement = Some(parse_move(
                node.get("move")
                    .ok_or_else(|| error(context, &format!("{node_context}: move is required.")))?,
                context,
                &node_context,
            )?);
            let role = Some(parse_role(node.get("role"), context, &node_context)?);
            let nags = parse_nags(node.get("nags"), context, &node_context)?;
            (parent, movement, role, nags)
        };

        pending_nodes.push(PendingNode {
            id: id.clone(),
            parent,
            movement,
            fen,
            position,
            role,
            comment,
            content,
            nags,
            children,
        });
    }

    let mut resolved_children = vec![Vec::new(); pending_nodes.len()];
    let mut child_owner: Vec<Option<AnalysisNodeIndex>> = vec![None; pending_nodes.len()];
    for (parent_number, node) in pending_nodes.iter().enumerate() {
        let parent_index = AnalysisNodeIndex(parent_number);
        for child_id in &node.children {
            let child_index = index_by_id.get(child_id).copied().ok_or_else(|| {
                error(
                    context,
                    &format!(
                        "analysis node {:?}: child {child_id:?} does not name an existing node.",
                        node.id
                    ),
                )
            })?;
            if child_index == root {
                return Err(error(
                    context,
                    &format!(
                        "analysis node {:?}: root node {root_id:?} cannot appear as a child.",
                        node.id
                    ),
                ));
            }
            if let Some(previous_parent) = child_owner[child_index.0] {
                return Err(error(
                    context,
                    &format!(
                        "analysis node {child_id:?} appears under both {:?} and {:?}.",
                        pending_nodes[previous_parent.0].id, node.id
                    ),
                ));
            }
            child_owner[child_index.0] = Some(parent_index);
            resolved_children[parent_number].push(child_index);
        }
    }

    for (index, node) in pending_nodes.iter().enumerate() {
        let node_index = AnalysisNodeIndex(index);
        if node_index == root {
            continue;
        }
        let parent_id = node.parent.as_deref().expect("non-root parent validated");
        let declared_parent = index_by_id.get(parent_id).copied().ok_or_else(|| {
            error(
                context,
                &format!(
                    "analysis node {:?}: parent {parent_id:?} does not name an existing node.",
                    node.id
                ),
            )
        })?;
        if child_owner[index] != Some(declared_parent) {
            return Err(error(
                context,
                &format!(
                    "analysis node {:?}: parent and children links must agree in both directions.",
                    node.id
                ),
            ));
        }
    }

    validate_connected_acyclic(&resolved_children, root, context)?;

    let mut nodes = Vec::with_capacity(pending_nodes.len());
    for (index, node) in pending_nodes.into_iter().enumerate() {
        let parent = node
            .parent
            .as_deref()
            .and_then(|parent_id| index_by_id.get(parent_id).copied());
        let content = resolve_spans(
            node.content,
            &index_by_id,
            root,
            context,
            &format!("analysis node {:?} content", node.id),
        )?;
        nodes.push(AnalysisNode {
            id: node.id,
            parent,
            movement: node.movement,
            fen: node.fen,
            position: node.position,
            role: node.role,
            comment: node.comment,
            content,
            nags: node.nags,
            children: resolved_children[index].clone(),
        });
    }

    let main_line = project_main_line(&nodes, root, context)?;
    let (main_line_ply_by_node, nearest_main_line_ancestor_by_node) =
        build_main_line_lookups(&nodes, &main_line);

    Ok(AnalysisTree {
        version: 1,
        root,
        nodes,
        index_by_id,
        main_line,
        main_line_ply_by_node,
        nearest_main_line_ancestor_by_node,
    })
}

fn validate_connected_acyclic(
    children: &[Vec<AnalysisNodeIndex>],
    root: AnalysisNodeIndex,
    context: &str,
) -> Result<(), String> {
    let mut indegree = vec![0usize; children.len()];
    for child_list in children {
        for child in child_list {
            indegree[child.0] += 1;
        }
    }

    let mut queue = VecDeque::new();
    for (index, degree) in indegree.iter().copied().enumerate() {
        if degree == 0 {
            queue.push_back(AnalysisNodeIndex(index));
        }
    }

    if queue.len() != 1 || queue.front().copied() != Some(root) {
        return Err(error(
            context,
            "analysis graph must be connected from its root and acyclic.",
        ));
    }

    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        for child in &children[node.0] {
            indegree[child.0] -= 1;
            if indegree[child.0] == 0 {
                queue.push_back(*child);
            }
        }
    }

    if visited != children.len() {
        return Err(error(
            context,
            "analysis graph must be connected from its root and acyclic.",
        ));
    }
    Ok(())
}

fn project_main_line(
    nodes: &[AnalysisNode],
    root: AnalysisNodeIndex,
    context: &str,
) -> Result<Vec<AnalysisNodeIndex>, String> {
    let mut main_line = vec![root];
    let mut cursor = root;

    loop {
        let mut main_child = None;
        for child in &nodes[cursor.0].children {
            if nodes[child.0].role == Some(AnalysisRole::Main)
                && main_child.replace(*child).is_some()
            {
                return Err(error(
                    context,
                    "analysis main path may contain at most one main child at each node.",
                ));
            }
        }

        let Some(next) = main_child else {
            break;
        };
        main_line.push(next);
        cursor = next;
    }

    Ok(main_line)
}

fn validate_puzzle_main_path(
    tree: &AnalysisTree,
    solution: &[String],
    context: &str,
) -> Result<(), String> {
    let projected = tree
        .main_line
        .iter()
        .skip(1)
        .map(|index| {
            tree.nodes[index.0]
                .movement
                .as_ref()
                .expect("non-root analysis node has a move")
                .uci
                .clone()
        })
        .collect::<Vec<_>>();

    if projected != solution {
        return Err(error(
            context,
            "analysis main-path UCI sequence must exactly match legacy solution.",
        ));
    }

    validate_main_role_membership(tree, context)
}

fn validate_review_main_path(tree: &AnalysisTree, context: &str) -> Result<(), String> {
    if tree.main_line.len() <= 1 {
        return Err(error(
            context,
            "analysis main path must contain at least one main move.",
        ));
    }
    validate_main_role_membership(tree, context)
}

fn validate_main_role_membership(tree: &AnalysisTree, context: &str) -> Result<(), String> {
    let main_nodes = tree.main_line.iter().copied().collect::<HashSet<_>>();
    for (index, node) in tree.nodes.iter().enumerate() {
        let node_index = AnalysisNodeIndex(index);
        if node_index != tree.root
            && node.role == Some(AnalysisRole::Main)
            && !main_nodes.contains(&node_index)
        {
            return Err(error(
                context,
                &format!(
                    "analysis node {:?}: main role is only valid on the projected main path.",
                    node.id
                ),
            ));
        }
    }
    Ok(())
}

fn build_main_line_lookups(
    nodes: &[AnalysisNode],
    main_line: &[AnalysisNodeIndex],
) -> (Vec<Option<usize>>, Vec<AnalysisNodeIndex>) {
    let root = main_line[0];
    let mut main_line_ply_by_node = vec![None; nodes.len()];
    for (ply, index) in main_line.iter().copied().enumerate() {
        main_line_ply_by_node[index.0] = Some(ply);
    }

    let mut nearest_main_line_ancestor_by_node = vec![root; nodes.len()];
    for index in 0..nodes.len() {
        let mut cursor = AnalysisNodeIndex(index);
        while main_line_ply_by_node[cursor.0].is_none() {
            cursor = nodes[cursor.0]
                .parent
                .expect("connected non-root analysis node has a parent");
        }
        nearest_main_line_ancestor_by_node[index] = cursor;
    }

    (main_line_ply_by_node, nearest_main_line_ancestor_by_node)
}

fn parse_move(value: &Value, context: &str, node_context: &str) -> Result<AnalysisMove, String> {
    let object = value
        .as_object()
        .ok_or_else(|| error(context, &format!("{node_context}: move must be an object.")))?;
    if object.len() != 2 || !object.contains_key("uci") || !object.contains_key("san") {
        return Err(error(
            context,
            &format!("{node_context}: move must contain exactly the uci and san fields."),
        ));
    }

    let uci = required_string(object, "uci", context, &format!("{node_context} move"))?;
    let parsed_uci = parse_uci_move(&uci).map_err(|uci_error| {
        error(
            context,
            &format!("{node_context}: invalid move UCI {uci:?}: {uci_error}."),
        )
    })?;
    let san = required_nonempty_string(object, "san", context, &format!("{node_context} move"))?;
    Ok(AnalysisMove {
        uci,
        parsed_uci,
        san,
    })
}

fn parse_role(
    value: Option<&Value>,
    context: &str,
    node_context: &str,
) -> Result<AnalysisRole, String> {
    match value {
        None => Ok(AnalysisRole::Sideline),
        Some(Value::String(role)) => match role.as_str() {
            "main" => Ok(AnalysisRole::Main),
            "alternative" => Ok(AnalysisRole::Alternative),
            "sideline" => Ok(AnalysisRole::Sideline),
            _ => Err(error(
                context,
                &format!("{node_context}: role must be main, alternative, or sideline."),
            )),
        },
        Some(_) => Err(error(
            context,
            &format!("{node_context}: role must be a string."),
        )),
    }
}

fn parse_nags(
    value: Option<&Value>,
    context: &str,
    node_context: &str,
) -> Result<Vec<Nag>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| error(context, &format!("{node_context}: nags must be an array.")))?;
    values
        .iter()
        .map(|nag| {
            nag.as_u64().map(Nag).ok_or_else(|| {
                error(
                    context,
                    &format!("{node_context}: nags must contain non-negative integers."),
                )
            })
        })
        .collect()
}

fn parse_pending_spans(value: &Value, context: &str) -> Result<Vec<PendingTextSpan>, String> {
    let spans = value
        .as_array()
        .ok_or_else(|| format!("{context}: structured content must be an array."))?;
    let mut parsed = Vec::with_capacity(spans.len());
    for (index, span) in spans.iter().enumerate() {
        let span_context = format!("{context} span {}", index + 1);
        let object = span
            .as_object()
            .ok_or_else(|| format!("{span_context}: span must be an object."))?;
        let kind = object
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{span_context}: type must be a string."))?;

        match kind {
            "text" => {
                if object.len() != 2 || !object.contains_key("text") {
                    return Err(format!(
                        "{span_context}: text span must contain exactly type and text."
                    ));
                }
                let text = object
                    .get("text")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| format!("{span_context}: text must be a non-empty string."))?;
                parsed.push(PendingTextSpan::Text(text.to_owned()));
            }
            "move_ref" => {
                if object.len() < 2
                    || object.len() > 3
                    || !object.contains_key("node")
                    || object
                        .keys()
                        .any(|key| !matches!(key.as_str(), "type" | "node" | "label"))
                {
                    return Err(format!(
                        "{span_context}: move_ref span may contain only type, node, and optional label."
                    ));
                }
                let node = object
                    .get("node")
                    .and_then(Value::as_str)
                    .filter(|node| !node.is_empty())
                    .ok_or_else(|| format!("{span_context}: node must be a non-empty string."))?
                    .to_owned();
                let label = match object.get("label") {
                    None => None,
                    Some(Value::String(label)) if !label.is_empty() => Some(label.clone()),
                    Some(_) => {
                        return Err(format!(
                            "{span_context}: label must be a non-empty string when present."
                        ));
                    }
                };
                parsed.push(PendingTextSpan::MoveRef { node, label });
            }
            _ => {
                return Err(format!(
                    "{span_context}: unsupported span type {kind:?}; expected text or move_ref."
                ));
            }
        }
    }
    Ok(parsed)
}

fn resolve_spans_with_tree(
    pending: Vec<PendingTextSpan>,
    analysis: Option<&AnalysisTree>,
    context: &str,
) -> Result<Vec<AnalysisTextSpan>, String> {
    let mut spans = Vec::with_capacity(pending.len());
    for span in pending {
        match span {
            PendingTextSpan::Text(text) => spans.push(AnalysisTextSpan::Text(text)),
            PendingTextSpan::MoveRef { node, label } => {
                let analysis = analysis.ok_or_else(|| {
                    format!("{context}: move_ref target {node:?} requires an analysis tree.")
                })?;
                let target = analysis.node_index(&node).ok_or_else(|| {
                    format!("{context}: move_ref target {node:?} does not exist.")
                })?;
                if target == analysis.root_index() {
                    return Err(format!(
                        "{context}: move_ref target {node:?} must not reference the root."
                    ));
                }
                spans.push(AnalysisTextSpan::MoveRef {
                    node: target,
                    label,
                });
            }
        }
    }
    Ok(spans)
}

fn resolve_spans(
    pending: Vec<PendingTextSpan>,
    index_by_id: &HashMap<String, AnalysisNodeIndex>,
    root: AnalysisNodeIndex,
    context: &str,
    span_context: &str,
) -> Result<Vec<AnalysisTextSpan>, String> {
    let mut spans = Vec::with_capacity(pending.len());
    for span in pending {
        match span {
            PendingTextSpan::Text(text) => spans.push(AnalysisTextSpan::Text(text)),
            PendingTextSpan::MoveRef { node, label } => {
                let target = index_by_id.get(&node).copied().ok_or_else(|| {
                    error(
                        context,
                        &format!("{span_context}: move_ref target {node:?} does not exist."),
                    )
                })?;
                if target == root {
                    return Err(error(
                        context,
                        &format!(
                            "{span_context}: move_ref target {node:?} must not reference root."
                        ),
                    ));
                }
                spans.push(AnalysisTextSpan::MoveRef {
                    node: target,
                    label,
                });
            }
        }
    }
    Ok(spans)
}

fn optional_string_array(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
    owner: &str,
) -> Result<Vec<String>, String> {
    let Some(value) = object.get(key) else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| error(context, &format!("{owner}: {key} must be an array.")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    error(
                        context,
                        &format!("{owner}: {key} must contain non-empty strings."),
                    )
                })
        })
        .collect()
}

fn optional_string(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
    owner: &str,
) -> Result<Option<String>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(error(
            context,
            &format!("{owner}: {key} must be a string when present."),
        )),
    }
}

fn required_nonempty_string(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
    owner: &str,
) -> Result<String, String> {
    required_string(object, key, context, owner).and_then(|value| {
        if value.is_empty() {
            Err(error(
                context,
                &format!("{owner}: {key} must be a non-empty string."),
            ))
        } else {
            Ok(value)
        }
    })
}

fn required_string(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
    owner: &str,
) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| error(context, &format!("{owner}: {key} must be a string.")))
}

fn required_u64(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
    owner: &str,
) -> Result<u64, String> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        error(
            context,
            &format!("{owner}: {key} must be a non-negative integer."),
        )
    })
}

fn error(context: &str, message: &str) -> String {
    format!("{context}: {message}")
}
