//! Deterministic paginated rendering for rich analysis browsing.

use crate::move_list::{draw_page_control, PageWindow};
use crate::{Gray8, Rect};
use chess_model::{AnalysisNodeIndex, AnalysisTextSpan, AnalysisTree, Color};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisHit {
    AnalysisMove(AnalysisNodeIndex),
    AnalysisPreviousPage,
    AnalysisNextPage,
}

#[cfg(test)]
const WHITE: u8 = 255;

#[derive(Clone, Copy, Debug)]
pub struct AnalysisView<'a> {
    pub tree: &'a AnalysisTree,
    pub leading_content: &'a [AnalysisTextSpan],
    pub selected: Option<AnalysisNodeIndex>,
    pub page: usize,
    pub focus: Option<AnalysisNodeIndex>,
}

pub use crate::move_flow::{FlowChip as GenericMoveChip, MoveChipSource as AnalysisMoveChipSource};
pub type AnalysisMoveChip = GenericMoveChip<AnalysisNodeIndex>;
use crate::move_flow::{draw_line, FlowBuilder, FlowDocument, FlowFragmentKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisPanelOutput<T = AnalysisHit> {
    target: std::marker::PhantomData<T>,
    pub page: usize,
    pub page_count: usize,
    pub move_chips: Vec<AnalysisMoveChip>,
    pub previous_page: Option<Rect>,
    pub next_page: Option<Rect>,
}

impl<T: From<AnalysisHit>> AnalysisPanelOutput<T> {
    pub fn map_targets<U>(self) -> AnalysisPanelOutput<U> {
        AnalysisPanelOutput {
            target: std::marker::PhantomData,
            page: self.page,
            page_count: self.page_count,
            move_chips: self.move_chips,
            previous_page: self.previous_page,
            next_page: self.next_page,
        }
    }
    pub fn hit_test(&self, x: u32, y: u32) -> Option<T> {
        self.hit_test_neutral(x, y).map(T::from)
    }
    fn hit_test_neutral(&self, x: u32, y: u32) -> Option<AnalysisHit> {
        if self.previous_page.is_some_and(|rect| rect.contains(x, y)) {
            return Some(AnalysisHit::AnalysisPreviousPage);
        }
        if self.next_page.is_some_and(|rect| rect.contains(x, y)) {
            return Some(AnalysisHit::AnalysisNextPage);
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
    Target(AnalysisHit),
    Ambiguous,
}

fn hit_move_chips(chips: &[AnalysisMoveChip], x: u32, y: u32, padded: bool) -> MoveHit {
    let mut hit = None;
    for chip in chips {
        let rect = if padded { chip.hit_rect } else { chip.rect };
        if !rect.contains(x, y) {
            continue;
        }
        let target = AnalysisHit::AnalysisMove(chip.node);
        match hit {
            Some(existing) if existing != target => return MoveHit::Ambiguous,
            Some(_) => {}
            None => hit = Some(target),
        }
    }

    hit.map_or(MoveHit::None, MoveHit::Target)
}

pub fn draw_analysis_panel(
    frame: &mut Gray8,
    view: AnalysisView<'_>,
    bounds: Rect,
    content: Rect,
    scale: u32,
) -> Option<AnalysisPanelOutput> {
    let analysis = view.tree;
    let document = build_document(
        view.leading_content,
        analysis,
        view.selected,
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
    let page_count = PageWindow::new(document.lines.len(), rows_per_page, 0).page_count;
    let requested_page = view.page.min(page_count.saturating_sub(1));
    let page = view
        .focus
        .and_then(|focus| {
            document.lines.iter().position(|line| {
                line.fragments.iter().any(|fragment| {
                    matches!(
                        &fragment.kind,
                        FlowFragmentKind::Chip {
                            node,
                            source: AnalysisMoveChipSource::TreeMove,
                            ..
                        } if *node == focus
                    )
                })
            })
        })
        .map(|line| line / rows_per_page)
        .unwrap_or(requested_page)
        .min(page_count.saturating_sub(1));

    let window = PageWindow::new(document.lines.len(), rows_per_page, page);
    let start = window.range.start;
    let end = window.range.end;
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

    debug_assert!(bounds.contains_rect(content));
    Some(AnalysisPanelOutput {
        target: std::marker::PhantomData,
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
) -> FlowDocument<AnalysisNodeIndex> {
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
                    builder.pending_space = true;
                    push_spans(&mut builder, 0, &node.content, analysis, selected);
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
    builder: &mut FlowBuilder<AnalysisNodeIndex>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::measure_text_regular;
    // Read fixture data through the generic model rather than product parsers.
    struct Fixture {
        analysis: Option<AnalysisTree>,
        description_content: Vec<AnalysisTextSpan>,
    }
    struct Book {
        puzzles: Vec<Fixture>,
    }
    fn parse_puzzle_file(bytes: &[u8]) -> Result<Book, String> {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let puzzles = value["puzzles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                let analysis = chess_model::analysis::parse_analysis_tree(
                    &p["analysis"],
                    p["fen"].as_str().unwrap(),
                    "root fen",
                    "fixture",
                )?;
                let description_content = if let Some(content) = p.get("description_content") {
                    chess_model::analysis::parse_text_content(content, Some(&analysis), "fixture")?
                } else {
                    Vec::new()
                };
                Ok(Fixture {
                    analysis: Some(analysis),
                    description_content,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Book { puzzles })
    }

    const BOOK: &[u8] = include_bytes!("../../../tests/fixtures/pgn-converter/valid-book.json");

    // Inspect the actual flow fragments consumed by drawing, including punctuation.
    // Spaces are normalized here so assertions are independent of physical wrapping.
    fn notation(document: &FlowDocument<AnalysisNodeIndex>) -> String {
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
    fn plain_move_record_matches_analysis_pixels_and_hit_rectangles() {
        let mut value: serde_json::Value = serde_json::from_slice(BOOK).unwrap();
        let tree = parse_puzzle_file(BOOK)
            .unwrap()
            .puzzles
            .remove(0)
            .analysis
            .unwrap();
        let ids: Vec<_> = std::iter::once(tree.root().id.clone())
            .chain(
                tree.main_line_nodes()
                    .iter()
                    .map(|n| tree.node(*n).unwrap().id.clone()),
            )
            .collect();
        let nodes = value["puzzles"][0]["analysis"]["nodes"]
            .as_array_mut()
            .unwrap();
        nodes.retain(|n| ids.contains(&n["id"].as_str().unwrap().to_owned()));
        for n in nodes {
            n.as_object_mut().unwrap().remove("content");
            n.as_object_mut().unwrap().remove("comment");
            n.as_object_mut().unwrap().remove("nags");
            if let Some(children) = n.get_mut("children").and_then(|c| c.as_array_mut()) {
                children.retain(|c| ids.contains(&c.as_str().unwrap().to_owned()));
            }
        }
        value["puzzles"][0]
            .as_object_mut()
            .unwrap()
            .remove("description_content");
        let tree = parse_puzzle_file(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .puzzles
            .remove(0)
            .analysis
            .unwrap();
        let labels: Vec<_> = tree
            .main_line_nodes()
            .iter()
            .skip(1)
            .map(|n| {
                tree.node(*n)
                    .unwrap()
                    .movement
                    .as_ref()
                    .unwrap()
                    .san
                    .as_str()
            })
            .collect();
        for width in [500, 1200] {
            let document = build_document(&[], &tree, None, width, 3);
            let body = Rect::new(
                10,
                10,
                width,
                document.lines.len() as u32 * document.row_height,
            );
            for selected in [0, 1, labels.len()] {
                let record = crate::move_flow::layout_move_record(&labels, selected, None, body, 3);
                let mut actual = Gray8::new(width + 20, body.bottom() + 10, WHITE);
                record.draw(&mut actual);
                let selected_node = selected
                    .checked_sub(1)
                    .and_then(|n| tree.main_line_nodes().get(n + 1))
                    .copied();
                let reference = build_document(&[], &tree, selected_node, width, 3);
                let mut expected = Gray8::new(width + 20, body.bottom() + 10, WHITE);
                let mut chips = Vec::new();
                for (slot, line) in reference.lines.iter().enumerate() {
                    draw_line(
                        &mut expected,
                        Rect::new(
                            body.x,
                            body.y + slot as u32 * reference.row_height,
                            width,
                            reference.row_height,
                        ),
                        line,
                        reference.chip_height,
                        3,
                        &mut chips,
                    );
                }
                assert_eq!(
                    actual.checksum64(),
                    expected.checksum64(),
                    "width={width}, selected={selected}"
                );
                assert_eq!(record.chips.len(), chips.len());
                for (actual, expected) in record.chips.iter().zip(chips) {
                    assert_eq!(
                        (actual.rect, actual.hit_rect, &actual.label, actual.selected),
                        (
                            expected.rect,
                            expected.hit_rect,
                            &expected.label,
                            expected.selected
                        )
                    );
                }
            }
        }
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
        assert!(notation(&document).starts_with("1. e4!!!? $99 Central move."));
    }

    #[test]
    fn review_comments_use_plain_prose_and_preserve_authored_braces() {
        let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../tests/fixtures/game-review/valid-standard.json"
        ))
        .unwrap();
        let nodes = value["games"][0]["analysis"]["nodes"]
            .as_array_mut()
            .unwrap();
        let node = nodes.iter_mut().find(|node| node["id"] == "n1").unwrap();
        node["comment"] = "Keep {authored braces} in prose.".into();
        node["content"] = serde_json::json!([
            {"type": "text", "text": "Keep {authored braces} in prose."}
        ]);
        let game = &value["games"][0];
        let tree = chess_model::analysis::parse_analysis_tree(
            &game["analysis"],
            game["fen"].as_str().unwrap(),
            "root fen",
            "fixture",
        )
        .unwrap();
        let document = build_document(&[], &tree, None, 1800, 3);
        let text = notation(&document);
        assert!(text.contains("Keep {authored braces} in prose."));
        assert_eq!(text.matches('{').count(), 1);
        assert_eq!(text.matches('}').count(), 1);
        assert!(text.contains('(') && text.contains(')'));
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
                "1. e4! Central move. Compare 1...e5 with e2e4 in plain text. ",
                "1... e5 ( 1... c5 Sicilian alternative. 2. Nf3 Nc6 ",
                "( 2... d6 Nested sideline. ) ) 2. Nf3 Nc6"
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
        assert_eq!(notation(&document), "1... h1=Q+ Promotion comment with plain h1=Q+. ( 1... h1=N Underpromotion sideline. ) 2. Ka2");
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
        assert!(text.contains("2. Nf3 Nc6 ( 2... e6 ) ( 2... d6 Nested sideline. ) )"));
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
        assert!(!text.contains(['{', '}']));
        let mut builder = FlowBuilder::<AnalysisNodeIndex>::new(180, 3);
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
        let panel: AnalysisPanelOutput = AnalysisPanelOutput {
            target: std::marker::PhantomData,
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
