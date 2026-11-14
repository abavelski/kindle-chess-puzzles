//! Shared inline movetext flow used by analysis and Play.
use crate::move_list::{draw_move_chip, PageWindow};
use crate::{
    font::{
        draw_text_bold_vertically_centered, draw_text_regular_vertically_centered,
        measure_text_bold, measure_text_regular, text_line_height,
    },
    Gray8, Rect,
};
const INK: u8 = 0;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoveChipSource {
    TreeMove,
    InlineReference,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowChip<T> {
    pub rect: Rect,
    pub hit_rect: Rect,
    pub node: T,
    pub source: MoveChipSource,
    pub label: String,
    pub selected: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct FlowDocument<T> {
    pub(crate) lines: Vec<FlowLine<T>>,
    pub(crate) row_height: u32,
    pub(crate) chip_height: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct FlowLine<T> {
    pub(crate) used: u32,
    pub(crate) fragments: Vec<FlowFragment<T>>,
}

#[derive(Clone, Debug)]
pub(crate) struct FlowFragment<T> {
    pub(crate) x: u32,
    pub(crate) width: u32,
    pub(crate) kind: FlowFragmentKind<T>,
}

#[derive(Clone, Debug)]
pub(crate) enum FlowFragmentKind<T> {
    Text {
        text: String,
        bold: bool,
    },
    Chip {
        node: T,
        source: MoveChipSource,
        label: String,
        selected: bool,
    },
}

pub(crate) struct FlowBuilder<T> {
    width: u32,
    scale: u32,
    row_height: u32,
    chip_height: u32,
    chip_padding_x: u32,
    space_width: u32,
    lines: Vec<FlowLine<T>>,
    current: Option<FlowLine<T>>,
    pub(crate) pending_space: bool,
}

impl<T: Copy> FlowBuilder<T> {
    pub(crate) fn new(width: u32, scale: u32) -> Self {
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

    pub(crate) fn finish(mut self) -> FlowDocument<T> {
        self.force_break();
        FlowDocument {
            lines: self.lines,
            row_height: self.row_height,
            chip_height: self.chip_height,
        }
    }

    pub(crate) fn force_break(&mut self) {
        if let Some(line) = self.current.take() {
            if !line.fragments.is_empty() {
                self.lines.push(line);
            }
        }
        self.pending_space = false;
    }

    pub(crate) fn push_text(&mut self, indent: u32, text: &str, bold: bool) {
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

    pub(crate) fn push_word(&mut self, indent: u32, word: &str, bold: bool) {
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

    pub(crate) fn push_chip(
        &mut self,
        indent: u32,
        node: T,
        label: &str,
        source: MoveChipSource,
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

    pub(crate) fn push_fragment(
        &mut self,
        indent: u32,
        requested_width: u32,
        kind: FlowFragmentKind<T>,
    ) {
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

    pub(crate) fn ensure_line(&mut self, indent: u32) {
        if self.current.is_none() {
            self.current = Some(FlowLine {
                used: indent,
                fragments: Vec::new(),
            });
        }
    }
}

pub(crate) fn draw_line<T: Copy>(
    frame: &mut Gray8,
    row: Rect,
    line: &FlowLine<T>,
    chip_height: u32,
    scale: u32,
    move_chips: &mut Vec<FlowChip<T>>,
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
                label, selected, ..
            } => {
                let chip = position_chip(row, fragment, chip_height, scale).expect("chip fragment");
                draw_move_chip(frame, chip.rect, label, *selected, scale);
                move_chips.push(chip);
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

fn position_chip<T: Copy>(
    row: Rect,
    fragment: &FlowFragment<T>,
    height: u32,
    scale: u32,
) -> Option<FlowChip<T>> {
    let FlowFragmentKind::Chip {
        node,
        source,
        label,
        selected,
    } = &fragment.kind
    else {
        return None;
    };
    let rect = Rect::new(
        row.x + fragment.x,
        row.y + row.height.saturating_sub(height) / 2,
        fragment.width,
        height,
    );
    Some(FlowChip {
        rect,
        hit_rect: padded_move_hit_rect(rect, row, scale),
        node: *node,
        source: *source,
        label: label.clone(),
        selected: *selected,
    })
}

/// A paged main-line move record using the same flow and painting as rich analysis.
#[derive(Clone, Debug)]
pub struct MoveRecordPage {
    pub window: PageWindow,
    pub chips: Vec<FlowChip<usize>>,
    document: FlowDocument<usize>,
    body: Rect,
    scale: u32,
}
impl MoveRecordPage {
    pub fn draw(&self, frame: &mut Gray8) {
        for (slot, line) in self.document.lines[self.window.range.clone()]
            .iter()
            .enumerate()
        {
            draw_line(
                frame,
                self.row(slot),
                line,
                self.document.chip_height.min(self.row(slot).height),
                self.scale,
                &mut Vec::new(),
            );
        }
    }
    fn row(&self, slot: usize) -> Rect {
        let height = self.document.row_height.min(self.body.height.max(1));
        Rect::new(
            self.body.x,
            self.body.y + slot as u32 * height,
            self.body.width,
            height,
        )
    }
}
/// Labels are SAN (or UCI for older servers), indexed from ply one.
pub fn layout_move_record(
    labels: &[&str],
    selected: usize,
    requested_page: Option<usize>,
    body: Rect,
    scale: u32,
) -> MoveRecordPage {
    let mut builder = FlowBuilder::new(body.width, scale);
    for (i, label) in labels.iter().enumerate() {
        if i % 2 == 0 {
            builder.push_text(0, &format!("{}. ", i / 2 + 1), false);
        }
        builder.push_chip(0, i + 1, label, MoveChipSource::TreeMove, selected == i + 1);
        builder.pending_space = true;
    }
    let document = builder.finish();
    let rows = (body.height / document.row_height.min(body.height.max(1))).max(1) as usize;
    let focused = document
        .lines
        .iter()
        .position(|line| {
            line.fragments
                .iter()
                .any(|f| matches!(&f.kind, FlowFragmentKind::Chip {node,..} if *node == selected))
        })
        .unwrap_or(0)
        / rows;
    let window = PageWindow::new(
        document.lines.len(),
        rows,
        requested_page.unwrap_or(focused),
    );
    let mut page = MoveRecordPage {
        window,
        chips: Vec::new(),
        document,
        body,
        scale,
    };
    for (slot, line) in page.document.lines[page.window.range.clone()]
        .iter()
        .enumerate()
    {
        let row = page.row(slot);
        for f in &line.fragments {
            if let Some(chip) =
                position_chip(row, f, page.document.chip_height.min(row.height), scale)
            {
                page.chips.push(chip);
            }
        }
    }
    page
}
