//! Deterministic paginated rendering for rich analysis browsing.

use crate::{
    font::{
        draw_text_bold_vertically_centered, draw_text_centered,
        draw_text_regular_vertically_centered, measure_text_bold, measure_text_regular,
        text_line_height,
    },
    Gray8, HitTarget, Layout, Rect,
};
use chess_core::{AnalysisNodeIndex, AnalysisRole, AnalysisTextSpan, AnalysisTree, AppState};

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
    pub close: Rect,
    pub next_page: Option<Rect>,
}

impl AnalysisPanelOutput {
    pub fn hit_test(&self, x: u32, y: u32) -> Option<HitTarget> {
        if self
            .previous_page
            .is_some_and(|rect| rect.contains(x, y))
        {
            return Some(HitTarget::AnalysisPreviousPage);
        }
        if self.close.contains(x, y) {
            return Some(HitTarget::CloseAnalysis);
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
    line_height: u32,
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
    line_height: u32,
    row_height: u32,
    chip_height: u32,
    indent_step: u32,
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
            line_height,
            row_height: chip_height.saturating_add(4),
            chip_height,
            indent_step: line_height.saturating_div(2).saturating_add(6),
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
            line_height: self.line_height,
            row_height: self.row_height,
            chip_height: self.chip_height,
        }
    }

    fn indent_for_depth(&self, depth: usize) -> u32 {
        u32::try_from(depth)
            .unwrap_or(u32::MAX)
            .saturating_mul(self.indent_step)
            .min(self.width / 3)
    }

    fn content_indent(&self, depth: usize) -> u32 {
        self.indent_for_depth(depth)
            .saturating_add(self.indent_step)
            .min(self.width / 3)
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
    let title_height = document.line_height.saturating_add(2).min(content.height);
    let footer_height = document
        .row_height
        .min(content.height.saturating_sub(title_height));
    let footer = Rect::new(
        content.x,
        content.bottom().saturating_sub(footer_height),
        content.width,
        footer_height,
    );
    let body_y = content.y.saturating_add(title_height).saturating_add(gap);
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

    let title = Rect::new(content.x, content.y, content.width, title_height);
    draw_text_bold_vertically_centered(
        frame,
        title.x,
        title,
        &format!("ANALYSIS {}/{}", page + 1, page_count),
        scale,
        INK,
    );
    if title.height > 1 {
        frame.fill_rect(
            Rect::new(title.x, title.bottom().saturating_sub(1), title.width, 1),
            INK,
        );
    }

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
    let next_page = (page + 1 < page_count).then_some(controls[2]);
    if let Some(rect) = previous_page {
        draw_page_control(frame, rect, "< PAGE", scale);
    }
    draw_page_control(frame, controls[1], "CLOSE", scale);
    if let Some(rect) = next_page {
        draw_page_control(frame, rect, "PAGE >", scale);
    }

    debug_assert!(layout.status.contains_rect(content));
    Some(AnalysisPanelOutput {
        page,
        page_count,
        move_chips,
        previous_page,
        close: controls[1],
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
        builder.push_text(0, "DESCRIPTION", true);
        builder.force_break();
        push_spans(&mut builder, 0, description, analysis, selected);
        builder.force_break();
    }

    let root = analysis.root();
    if !root.content.is_empty() {
        builder.push_text(0, "START", true);
        builder.force_break();
        push_spans(&mut builder, 0, root.content.as_slice(), analysis, selected);
        builder.force_break();
    }

    let mut stack = root
        .children
        .iter()
        .rev()
        .copied()
        .map(|node| (node, 0usize))
        .collect::<Vec<_>>();
    while let Some((index, depth)) = stack.pop() {
        let Some(node) = analysis.node(index) else {
            continue;
        };
        let indent = builder.indent_for_depth(depth);
        let role = match node.role {
            Some(AnalysisRole::Main) => "MAIN",
            Some(AnalysisRole::Alternative) => "ALTERNATIVE",
            Some(AnalysisRole::Sideline) | None => "SIDELINE",
        };
        if selected == Some(index) {
            builder.push_text(indent, "SELECTED", true);
            builder.pending_space = true;
        }
        builder.push_text(indent, role, true);
        builder.pending_space = true;

        if let Some(movement) = node.movement.as_ref() {
            builder.push_chip(
                indent,
                index,
                &movement.san,
                AnalysisMoveChipSource::TreeMove,
                selected == Some(index),
            );
        }
        for nag in &node.nags {
            builder.push_text(indent, &format!(" ${}", nag.value()), false);
        }
        builder.force_break();

        if !node.content.is_empty() {
            let content_indent = builder.content_indent(depth);
            push_spans(
                &mut builder,
                content_indent,
                node.content.as_slice(),
                analysis,
                selected,
            );
            builder.force_break();
        }

        for child in node.children.iter().rev() {
            stack.push((*child, depth.saturating_add(1)));
        }
    }

    builder.finish()
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
    let right = rect
        .right()
        .saturating_add(padding)
        .min(row.right());
    Rect::new(x, row.y, right.saturating_sub(x), row.height)
}

fn draw_move_chip(frame: &mut Gray8, rect: Rect, label: &str, selected: bool, scale: u32) {
    let (background, foreground, border) = if selected {
        (INK, WHITE, 4)
    } else {
        (WHITE, INK, 2)
    };
    frame.fill_rect(rect, background);
    frame.stroke_rect(rect, border.min(rect.width).min(rect.height), INK);
    draw_text_centered(frame, rect.inset(4), label, scale, foreground);
}

fn split_footer(rect: Rect, gap: u32) -> [Rect; 3] {
    let available = rect.width.saturating_sub(gap.saturating_mul(2));
    let width = available / 3;
    let first = Rect::new(rect.x, rect.y, width, rect.height);
    let second_x = first.right().saturating_add(gap);
    let second = Rect::new(second_x, rect.y, width, rect.height);
    let third_x = second.right().saturating_add(gap);
    let third = Rect::new(
        third_x,
        rect.y,
        rect.right().saturating_sub(third_x),
        rect.height,
    );
    [first, second, third]
}

fn draw_page_control(frame: &mut Gray8, rect: Rect, label: &str, scale: u32) {
    frame.fill_rect(rect, WHITE);
    frame.stroke_rect(rect, 2.min(rect.width).min(rect.height), INK);
    draw_text_centered(frame, rect.inset(4), label, scale, INK);
}
