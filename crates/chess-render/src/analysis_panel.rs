//! Deterministic paginated rendering for rich analysis browsing.

use crate::{
    font::{
        draw_text_bold_vertically_centered, draw_text_centered,
        draw_text_regular_vertically_centered, measure_text_bold, measure_text_regular,
        text_line_height,
    },
    Gray8, HitTarget, Layout, Rect,
};
use chess_core::{AnalysisNodeIndex, AnalysisTextSpan, AnalysisTree, AppState, Color};

const WHITE: u8 = 255;
const INK: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisMoveChipSource {
    TreeMove,
    InlineReference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisMoveChip {
    pub rect: Rect,
    pub hit_rect: Rect,
    pub node: AnalysisNodeIndex,
    pub source: AnalysisMoveChipSource,
    pub label: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisPanelOutput {
    pub page: usize,
    pub page_count: usize,
    pub move_chips: Vec<AnalysisMoveChip>,
    pub previous_page: Option<Rect>,
    pub next_page: Option<Rect>,
}

impl AnalysisPanelOutput {
    pub fn hit_test(&self, x: u32, y: u32) -> Option<HitTarget> {
        if self.previous_page.is_some_and(|rect| rect.contains(x, y)) {
            return Some(HitTarget::AnalysisPreviousPage);
        }
        if self.next_page.is_some_and(|rect| rect.contains(x, y)) {
            return Some(HitTarget::AnalysisNextPage);
        }

        match hit_move_chips(&self.move_chips, x, y, false) {
            MoveHit::Target(target) => return Some(target),
            MoveHit::Ambiguous => return None,
            MoveHit::None => {}
        }

        match hit_move_chips(&self.move_chips, x, y, true) {
            MoveHit::Target(target) => Some(target),
            MoveHit::None | MoveHit::Ambiguous => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MoveHit {
    None,
    Target(HitTarget),
    Ambiguous,
}

fn hit_move_chips(chips: &[AnalysisMoveChip], x: u32, y: u32, padded: bool) -> MoveHit {
    let mut hit = None;
    for chip in chips {
        let rect = if padded { chip.hit_rect } else { chip.rect };
        if !rect.contains(x, y) {
            continue;
        }
        let target = HitTarget::AnalysisMove(chip.node);
        match hit {
            Some(existing) if existing != target => return MoveHit::Ambiguous,
            Some(_) => {}
            None => hit = Some(target),
        }
    }

    hit.map_or(MoveHit::None, MoveHit::Target)
}

#[derive(Clone, Debug)]
struct FlowDocument {
    lines: Vec<FlowLine>,
    row_height: u32,
    chip_height: u32,
}

#[derive(Clone, Debug)]
struct FlowLine {
    used: u32,
    fragments: Vec<FlowFragment>,
}

#[derive(Clone, Debug)]
struct FlowFragment {
    x: u32,
    width: u32,
    kind: FlowFragmentKind,
}

#[derive(Clone, Debug)]
enum FlowFragmentKind {
    Text {
        text: String,
        bold: bool,
    },
    Chip {
        node: AnalysisNodeIndex,
        source: AnalysisMoveChipSource,
        label: String,
        selected: bool,
    },
}

struct FlowBuilder {
    width: u32,
    scale: u32,
    row_height: u32,
    chip_height: u32,
    chip_padding_x: u32,
    space_width: u32,
    lines: Vec<FlowLine>,
    current: Option<FlowLine>,
    pending_space: bool,
}

impl FlowBuilder {
    fn new(width: u32, scale: u32) -> Self {
        let line_height = text_line_height(scale).max(1);
        let chip_height = line_height.saturating_add(8);
        Self {
            width: width.max(1),
            scale,
            row_height: chip_height.saturating_add(4),
            chip_height,
            chip_padding_x: scale.saturating_mul(4).max(8),
            space_width: measure_text_regular(" ", scale).max(scale),
            lines: Vec::new(),
            current: None,
            pending_space: false,
        }
    }

    fn finish(mut self) -> FlowDocument {
        self.force_break();
        FlowDocument {
            lines: self.lines,
            row_height: self.row_height,
            chip_height: self.chip_height,
        }
    }

    fn force_break(&mut self) {
        if let Some(line) = self.current.take() {
            if !line.fragments.is_empty() {
                self.lines.push(line);
            }
        }
        self.pending_space = false;
    }

    fn push_text(&mut self, indent: u32, text: &str, bold: bool) {
        let mut word = String::new();
        for character in text.chars() {
            if character == '\n' {
                if !word.is_empty() {
                    self.push_word(indent, &word, bold);
                    word.clear();
                }
                self.force_break();
            } else if character.is_whitespace() {
                if !word.is_empty() {
                    self.push_word(indent, &word, bold);
                    word.clear();
                }
                self.pending_space = true;
            } else {
                word.push(character);
            }
        }
        if !word.is_empty() {
            self.push_word(indent, &word, bold);
        }
    }

    fn push_word(&mut self, indent: u32, word: &str, bold: bool) {
        let width = if bold {
            measure_text_bold(word, self.scale)
        } else {
            measure_text_regular(word, self.scale)
        }
        .max(1);
        // Long prose words (URLs, for example) must wrap instead of being
        // assigned a clipped rectangle and silently losing their suffix.
        if width > self.width.saturating_sub(indent) {
            let available = self.width.saturating_sub(indent).max(1);
            let mut chunk = String::new();
            for character in word.chars() {
                let candidate = format!("{chunk}{character}");
                let measured = if bold {
                    measure_text_bold(&candidate, self.scale)
                } else {
                    measure_text_regular(&candidate, self.scale)
                };
                if measured > available && !chunk.is_empty() {
                    self.push_word(indent, &chunk, bold);
                    self.force_break();
                    chunk.clear();
                }
                chunk.push(character);
            }
            let final_width = if bold {
                measure_text_bold(&chunk, self.scale)
            } else {
                measure_text_regular(&chunk, self.scale)
            };
            // A single glyph may exceed a very small viewport; don't recurse.
            self.push_fragment(
                indent,
                final_width.min(available).max(1),
                FlowFragmentKind::Text { text: chunk, bold },
            );
            return;
        }
        self.push_fragment(
            indent,
            width,
            FlowFragmentKind::Text {
                text: word.to_owned(),
                bold,
            },
        );
    }

    fn push_chip(
        &mut self,
        indent: u32,
        node: AnalysisNodeIndex,
        label: &str,
        source: AnalysisMoveChipSource,
        selected: bool,
    ) {
        let width = measure_text_bold(label, self.scale)
            .saturating_add(self.chip_padding_x.saturating_mul(2))
            .max(self.chip_height);
        self.push_fragment(
            indent,
            width,
            FlowFragmentKind::Chip {
                node,
                source,
                label: label.to_owned(),
                selected,
            },
        );
    }

    fn push_fragment(&mut self, indent: u32, requested_width: u32, kind: FlowFragmentKind) {
        let indent = indent.min(self.width.saturating_sub(1));
        self.ensure_line(indent);

        let has_fragments = self
            .current
            .as_ref()
            .is_some_and(|line| !line.fragments.is_empty());
        let space = if self.pending_space && has_fragments {
            self.space_width
        } else {
            0
        };
        let used = self.current.as_ref().map_or(indent, |line| line.used);
        let remaining = self.width.saturating_sub(used);
        if has_fragments && space.saturating_add(requested_width) > remaining {
            self.force_break();
            self.ensure_line(indent);
        }

        let has_fragments = self
            .current
            .as_ref()
            .is_some_and(|line| !line.fragments.is_empty());
        let space = if self.pending_space && has_fragments {
            self.space_width
        } else {
            0
        };
        let line = self.current.as_mut().expect("line was ensured");
        let x = line.used.saturating_add(space).min(self.width);
        let width = requested_width.min(self.width.saturating_sub(x));
        if width == 0 {
            self.pending_space = false;
            return;
        }
        line.fragments.push(FlowFragment { x, width, kind });
        line.used = x.saturating_add(width);
        self.pending_space = false;
    }

    fn ensure_line(&mut self, indent: u32) {
        if self.current.is_none() {
            self.current = Some(FlowLine {
                used: indent,
                fragments: Vec::new(),
            });
        }
    }
}

pub(crate) fn draw_analysis_panel(
    frame: &mut Gray8,
    state: &AppState,
    layout: Layout,
    content: Rect,
    scale: u32,
) -> Option<AnalysisPanelOutput> {
    let puzzle = state.active_puzzle();
    let analysis = puzzle.analysis.as_ref()?;
    let selected = state.selected_analysis_node();
    let document = build_document(
        puzzle.description_content.as_slice(),
        analysis,
        selected,
        content.width,
        scale,
    );

    let gap = scale.saturating_mul(2).max(4);
    let footer_height = document.row_height.min(content.height);
    let footer = Rect::new(
        content.x,
        content.bottom().saturating_sub(footer_height),
        content.width,
        footer_height,
    );
    let body_y = content.y;
    let body_bottom = footer.y.saturating_sub(gap);
    let body = Rect::new(
        content.x,
        body_y,
        content.width,
        body_bottom.saturating_sub(body_y),
    );

    let row_height = document.row_height.min(body.height.max(1));
    let rows_per_page = usize::try_from(body.height / row_height)
        .unwrap_or(0)
        .max(1);
    let page_count = document.lines.len().div_ceil(rows_per_page).max(1);
    let page = state.analysis_page().min(page_count.saturating_sub(1));

    let start = page.saturating_mul(rows_per_page);
    let end = start
        .saturating_add(rows_per_page)
        .min(document.lines.len());
    let mut move_chips = Vec::new();
    for (slot, line) in document.lines[start..end].iter().enumerate() {
        let row_y = body.y.saturating_add(
            u32::try_from(slot)
                .unwrap_or(u32::MAX)
                .saturating_mul(row_height),
        );
        let row = Rect::new(body.x, row_y, body.width, row_height);
        draw_line(
            frame,
            row,
            line,
            document.chip_height.min(row.height),
            scale,
            &mut move_chips,
        );
    }

    let controls = split_footer(footer, gap);
    let previous_page = (page > 0).then_some(controls[0]);
    let next_page = (page + 1 < page_count).then_some(controls[1]);
    if let Some(rect) = previous_page {
        draw_page_control(frame, rect, "< PAGE", scale);
    }
    if let Some(rect) = next_page {
        draw_page_control(frame, rect, "PAGE >", scale);
    }

    debug_assert!(layout.status.contains_rect(content));
    Some(AnalysisPanelOutput {
        page,
        page_count,
        move_chips,
        previous_page,
        next_page,
    })
}

fn build_document(
    description: &[AnalysisTextSpan],
    analysis: &AnalysisTree,
    selected: Option<AnalysisNodeIndex>,
    width: u32,
    scale: u32,
) -> FlowDocument {
    let mut builder = FlowBuilder::new(width, scale);

    if !description.is_empty() {
        push_spans(&mut builder, 0, description, analysis, selected);
        builder.force_break();
    }

    let root = analysis.root();
    if !root.content.is_empty() {
        push_spans(&mut builder, 0, &root.content, analysis, selected);
        builder.force_break();
    }

    // A position emits its first move, then ordered sibling RAVs, then the
    // first move's continuation. Roles are grading metadata, not PGN structure.
    // Use a work stack so deeply nested input never consumes the call stack.
    enum Work {
        Position(AnalysisNodeIndex, bool),
        Branch(AnalysisNodeIndex),
        Move(AnalysisNodeIndex, bool),
        Open,
        Close,
    }
    let mut stack = vec![Work::Position(analysis.root_index(), true)];
    while let Some(work) = stack.pop() {
        match work {
            Work::Position(parent, force_number) => {
                let children = &analysis.node(parent).expect("validated node").children;
                let Some((&first, variations)) = children.split_first() else {
                    continue;
                };
                let first_node = analysis.node(first).expect("validated child");
                stack.push(Work::Position(
                    first,
                    !variations.is_empty() || !first_node.content.is_empty(),
                ));
                for &sibling in variations.iter().rev() {
                    stack.push(Work::Close);
                    stack.push(Work::Branch(sibling));
                    stack.push(Work::Open);
                }
                stack.push(Work::Move(first, force_number));
            }
            Work::Branch(index) => {
                let node = analysis.node(index).expect("validated node");
                stack.push(Work::Position(index, !node.content.is_empty()));
                stack.push(Work::Move(index, true));
            }
            Work::Move(index, force_number) => {
                let node = analysis.node(index).expect("validated node");
                let parent = analysis.parent(index).expect("move has parent");
                let position = &parent.position;
                if position.active_color() == Color::White || force_number {
                    let suffix = if position.active_color() == Color::White {
                        "."
                    } else {
                        "..."
                    };
                    builder.push_text(
                        0,
                        &format!("{}{} ", position.fullmove_number(), suffix),
                        false,
                    );
                }
                let movement = node.movement.as_ref().expect("non-root move");
                let mut label = movement.san.clone();
                for symbol in node
                    .nags
                    .iter()
                    .filter_map(|nag| move_annotation(nag.value()))
                {
                    label.push_str(symbol);
                }
                builder.push_chip(
                    0,
                    index,
                    &label,
                    AnalysisMoveChipSource::TreeMove,
                    selected == Some(index),
                );
                for nag in &node.nags {
                    if move_annotation(nag.value()).is_none() {
                        builder.push_text(0, &format!(" ${}", nag.value()), false);
                    }
                }
                if !node.content.is_empty() {
                    builder.push_text(0, " {", false);
                    push_spans(&mut builder, 0, &node.content, analysis, selected);
                    builder.pending_space = false;
                    builder.push_text(0, "}", false);
                }
                builder.pending_space = true;
            }
            Work::Open => {
                builder.push_text(0, "(", false);
            }
            Work::Close => {
                builder.pending_space = false;
                builder.push_text(0, ")", false);
                builder.pending_space = true;
            }
        }
    }

    builder.finish()
}

fn move_annotation(code: u64) -> Option<&'static str> {
    match code {
        1 => Some("!"),
        2 => Some("?"),
        3 => Some("!!"),
        4 => Some("??"),
        5 => Some("!?"),
        6 => Some("?!"),
        _ => None,
    }
}

fn push_spans(
    builder: &mut FlowBuilder,
    indent: u32,
    spans: &[AnalysisTextSpan],
    analysis: &AnalysisTree,
    selected: Option<AnalysisNodeIndex>,
) {
    for span in spans {
        match span {
            AnalysisTextSpan::Text(text) => builder.push_text(indent, text, false),
            AnalysisTextSpan::MoveRef { node, label } => {
                let display = label
                    .as_deref()
                    .or_else(|| {
                        analysis
                            .node(*node)
                            .and_then(|target| target.movement.as_ref())
                            .map(|movement| movement.san.as_str())
                    })
                    .unwrap_or("?");
                builder.push_chip(
                    indent,
                    *node,
                    display,
                    AnalysisMoveChipSource::InlineReference,
                    selected == Some(*node),
                );
            }
        }
    }
}

fn draw_line(
    frame: &mut Gray8,
    row: Rect,
    line: &FlowLine,
    chip_height: u32,
    scale: u32,
    move_chips: &mut Vec<AnalysisMoveChip>,
) {
    for fragment in &line.fragments {
        let x = row.x.saturating_add(fragment.x);
        match &fragment.kind {
            FlowFragmentKind::Text { text, bold } => {
                if *bold {
                    draw_text_bold_vertically_centered(frame, x, row, text, scale, INK);
                } else {
                    draw_text_regular_vertically_centered(frame, x, row, text, scale, INK);
                }
            }
            FlowFragmentKind::Chip {
                node,
                source,
                label,
                selected,
            } => {
                let rect = Rect::new(
                    x,
                    row.y
                        .saturating_add(row.height.saturating_sub(chip_height) / 2),
                    fragment.width,
                    chip_height,
                );
                draw_move_chip(frame, rect, label, *selected, scale);
                move_chips.push(AnalysisMoveChip {
                    rect,
                    hit_rect: padded_move_hit_rect(rect, row, scale),
                    node: *node,
                    source: *source,
                    label: label.clone(),
                    selected: *selected,
                });
            }
        }
    }
}

fn padded_move_hit_rect(rect: Rect, row: Rect, scale: u32) -> Rect {
    let padding = scale.saturating_mul(2).max(4);
    let x = rect.x.saturating_sub(padding).max(row.x);
    let right = rect.right().saturating_add(padding).min(row.right());
    Rect::new(x, row.y, right.saturating_sub(x), row.height)
}

fn draw_move_chip(frame: &mut Gray8, rect: Rect, label: &str, selected: bool, scale: u32) {
    let (background, foreground) = if selected { (INK, WHITE) } else { (WHITE, INK) };
    frame.fill_rect(rect, background);
    draw_text_centered(frame, rect.inset(4), label, scale, foreground);
}

fn split_footer(rect: Rect, gap: u32) -> [Rect; 2] {
    let width = rect.width.saturating_sub(gap) / 2;
    let first = Rect::new(rect.x, rect.y, width, rect.height);
    let second_x = first.right().saturating_add(gap);
    [
        first,
        Rect::new(
            second_x,
            rect.y,
            rect.right().saturating_sub(second_x),
            rect.height,
        ),
    ]
}

fn draw_page_control(frame: &mut Gray8, rect: Rect, label: &str, scale: u32) {
    frame.fill_rect(rect, WHITE);
    frame.stroke_rect(rect, 2.min(rect.width).min(rect.height), INK);
    draw_text_centered(frame, rect.inset(4), label, scale, INK);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::parse_puzzle_file;

    const BOOK: &[u8] = include_bytes!("../../../tests/fixtures/pgn-converter/valid-book.json");

    // Inspect the actual flow fragments consumed by drawing, including punctuation.
    // Spaces are normalized here so assertions are independent of physical wrapping.
    fn notation(document: &FlowDocument) -> String {
        document
            .lines
            .iter()
            .flat_map(|line| &line.fragments)
            .map(|fragment| match &fragment.kind {
                FlowFragmentKind::Text { text, .. } => text.as_str(),
                FlowFragmentKind::Chip { label, .. } => label.as_str(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn tree_with_nags(nags: &[u64]) -> AnalysisTree {
        let mut value: serde_json::Value = serde_json::from_slice(BOOK).unwrap();
        value["puzzles"][0]["analysis"]["nodes"][1]["nags"] = serde_json::json!(nags);
        parse_puzzle_file(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .puzzles[0]
            .analysis
            .as_ref()
            .unwrap()
            .clone()
    }

    #[test]
    fn move_quality_annotations_are_attached_to_san_and_wrap_with_the_move() {
        for (code, symbol) in [
            (1, "!"),
            (2, "?"),
            (3, "!!"),
            (4, "??"),
            (5, "!?"),
            (6, "?!"),
        ] {
            let tree = tree_with_nags(&[code]);
            for width in [1800, 180] {
                let document = build_document(&[], &tree, None, width, 3);
                let labels: Vec<_> = document
                    .lines
                    .iter()
                    .flat_map(|line| &line.fragments)
                    .filter_map(|fragment| match &fragment.kind {
                        FlowFragmentKind::Chip { label, .. } => Some(label.as_str()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(labels[0], format!("e4{symbol}"));
                assert!(!notation(&document).contains(&format!("${code}")));
            }
        }
    }

    #[test]
    fn multiple_move_annotations_and_unknown_codes_preserve_their_information() {
        let tree = tree_with_nags(&[3, 5, 99]);
        let document = build_document(&[], &tree, None, 1800, 3);
        assert!(notation(&document).starts_with("1. e4!!!? $99 { Central move."));
    }

    #[test]
    fn book_movetext_serializes_ordered_nested_ravs_comments_nags_and_refs() {
        let book = parse_puzzle_file(BOOK).unwrap();
        let puzzle = &book.puzzles[0];
        let tree = puzzle.analysis.as_ref().unwrap();
        let document = build_document(&puzzle.description_content, tree, None, 1800, 3);
        assert_eq!(
            notation(&document),
            concat!(
                "Compare the main knight with c5 and plain Nd5/e2e4. ",
                "1. e4! { Central move. Compare 1...e5 with e2e4 in plain text. } ",
                "1... e5 ( 1... c5 { Sicilian alternative. } 2. Nf3 Nc6 ",
                "( 2... d6 { Nested sideline. } ) ) 2. Nf3 Nc6"
            )
        );
        assert!(document.lines.len() <= 6, "compact wrapped flow");
    }

    #[test]
    fn main_only_moves_share_lines_and_use_conventional_white_numbering() {
        let book = parse_puzzle_file(BOOK).unwrap();
        let tree = book.puzzles[2].analysis.as_ref().unwrap();
        let document = build_document(&[], tree, None, 1800, 3);
        assert_eq!(
            notation(&document),
            "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Bxc6 dxc6 5. O-O"
        );
        assert!(document.lines.len() <= 2);
        assert!(
            document.lines[0]
                .fragments
                .iter()
                .filter(|f| matches!(f.kind, FlowFragmentKind::Chip { .. }))
                .count()
                > 1
        );
    }

    #[test]
    fn black_start_numbering_and_root_prose_survive() {
        let book = parse_puzzle_file(BOOK).unwrap();
        let mut tree = book.puzzles[1].analysis.as_ref().unwrap().clone();
        let document = build_document(&[], &tree, None, 1800, 3);
        assert_eq!(notation(&document), "1... h1=Q+ { Promotion comment with plain h1=Q+. } ( 1... h1=N { Underpromotion sideline. } ) 2. Ka2");
        // Clone the parsed tree through JSON to change all starting counters coherently.
        let mut value: serde_json::Value = serde_json::from_slice(BOOK).unwrap();
        let puzzle = &mut value["puzzles"][1];
        puzzle["fen"] = puzzle["fen"]
            .as_str()
            .unwrap()
            .replace("0 1", "0 37")
            .into();
        for node in puzzle["analysis"]["nodes"].as_array_mut().unwrap() {
            let fen = node["fen"].as_str().unwrap();
            let mut fields = fen
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            fields[5] = (fields[5].parse::<u32>().unwrap() + 36).to_string();
            node["fen"] = fields.join(" ").into();
        }
        puzzle["analysis"]["nodes"][0]["comment"] = "Root prose.".into();
        let changed = parse_puzzle_file(&serde_json::to_vec(&value).unwrap()).unwrap();
        tree = changed.puzzles[1].analysis.as_ref().unwrap().clone();
        let document = build_document(&[], &tree, None, 1800, 3);
        assert!(notation(&document).starts_with("Root prose. 37... h1=Q+"));
        assert!(notation(&document).ends_with("38. Ka2"));
    }

    #[test]
    fn multiple_sibling_variations_follow_source_order() {
        let mut value: serde_json::Value = serde_json::from_slice(BOOK).unwrap();
        let nodes = value["puzzles"][0]["analysis"]["nodes"]
            .as_array_mut()
            .unwrap();
        let mut sibling = nodes.iter().find(|n| n["id"] == "n6").unwrap().clone();
        sibling["id"] = "sibling".into();
        sibling["move"]["san"] = "e6".into();
        sibling["move"]["uci"] = "e7e6".into();
        sibling["content"] = serde_json::json!([]);
        nodes.iter_mut().find(|n| n["id"] == "n4").unwrap()["children"] =
            serde_json::json!(["n5", "sibling", "n6"]);
        nodes.push(sibling);
        let book = parse_puzzle_file(&serde_json::to_vec(&value).unwrap()).unwrap();
        let tree = book.puzzles[0].analysis.as_ref().unwrap();
        let text = notation(&build_document(&[], tree, None, 1800, 3));
        assert!(text.contains("2. Nf3 Nc6 ( 2... e6 ) ( 2... d6 { Nested sideline. } ) )"));
    }
    #[test]
    fn overflow_wraps_all_punctuation_moves_and_long_comment_words_without_clipping() {
        let book = parse_puzzle_file(BOOK).unwrap();
        let tree = book.puzzles[0].analysis.as_ref().unwrap();
        let wide = build_document(&[], tree, None, 1800, 3);
        let narrow = build_document(&[], tree, None, 180, 3);
        assert_eq!(
            notation(&narrow),
            notation(&wide),
            "wrapping preserves token order"
        );
        let text = notation(&narrow);
        assert_eq!(text.matches('(').count(), 2);
        assert_eq!(text.matches(')').count(), 2);
        assert_eq!(text.matches('{').count(), text.matches('}').count());
        let mut builder = FlowBuilder::new(180, 3);
        let long_word = "explanation".repeat(20);
        builder.push_text(0, &long_word, false);
        let document = builder.finish();
        let mut recovered = String::new();
        for line in &document.lines {
            for fragment in &line.fragments {
                if let FlowFragmentKind::Text { text, .. } = &fragment.kind {
                    assert!(
                        measure_text_regular(text, 3) <= fragment.width,
                        "text must fit its fragment"
                    );
                    recovered.push_str(text);
                }
            }
        }
        assert_eq!(recovered, long_word);
        assert!(document.lines.len() > 1);
    }
    #[test]
    fn plain_move_looking_comment_fragments_are_inert() {
        let book = parse_puzzle_file(BOOK).unwrap();
        let tree = book.puzzles[0].analysis.as_ref().unwrap();
        let document = build_document(&[], tree, None, 1800, 3);
        let mut frame = Gray8::new(1800, 1000, WHITE);
        let mut chips = Vec::new();
        let mut inert_points = Vec::new();
        for (i, line) in document.lines.iter().enumerate() {
            let row = Rect::new(0, i as u32 * document.row_height, 1800, document.row_height);
            draw_line(&mut frame, row, line, document.chip_height, 3, &mut chips);
            for fragment in &line.fragments {
                if matches!(&fragment.kind, FlowFragmentKind::Text {text, ..} if text == "e2e4") {
                    inert_points.push((fragment.x + fragment.width / 2, row.y + row.height / 2));
                }
            }
        }
        assert!(!inert_points.is_empty());
        let panel = AnalysisPanelOutput {
            page: 0,
            page_count: 1,
            move_chips: chips,
            previous_page: None,
            next_page: None,
        };
        for (x, y) in inert_points {
            assert_eq!(panel.hit_test(x, y), None);
        }
    }
}
