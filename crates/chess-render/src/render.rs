//! Deterministic monochrome-first chess application renderer.

use crate::{
    analysis_panel::draw_analysis_panel,
    font::{
        draw_text_bold, draw_text_bold_vertically_centered, draw_text_centered, draw_wrapped_text,
        draw_wrapped_text_with_line_spacing, measure_text_bold,
    },
    pieces::draw_piece,
    AnalysisPanelOutput, DisplayMetrics, Gray8, HitTarget, Layout, LayoutError, Rect,
};
use chess_core::{AppState, BoardMode, Color, PieceKind, SolutionFeedback};

const WHITE: u8 = 255;
const INK: u8 = 0;
const LIGHT_SQUARE: u8 = 238;
const DARK_SQUARE: u8 = 184;
const SOFT_GRAY: u8 = 224;
const DISABLED_INK: u8 = 128;
const BOARD_FRAME_GRAY: u8 = 160;
const DESCRIPTION_EXTRA_LINE_SPACING: u32 = 6;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOutput {
    pub frame: Gray8,
    pub layout: Layout,
    pub damage: Vec<Rect>,
    pub analysis: Option<AnalysisPanelOutput>,
    pub analysis_entry: Option<Rect>,
}

impl RenderOutput {
    pub fn hit_test_app(&self, x: u32, y: u32, state: &AppState) -> Option<HitTarget> {
        if state.settings_open() {
            return self.layout.hit_test_app(x, y, state);
        }

        if state.analysis_browser_open()
            && !state.collection_picker_open()
            && !state.puzzle_goto_open()
        {
            if let Some(target) = self
                .analysis
                .as_ref()
                .and_then(|panel| panel.hit_test(x, y))
            {
                return Some(target);
            }
            if self.layout.board.contains(x, y) {
                return None;
            }
        }

        self.layout.hit_test_app(x, y, state)
    }
}

pub fn render(state: &AppState, metrics: DisplayMetrics) -> Result<RenderOutput, LayoutError> {
    let layout = Layout::new(metrics)?;
    let mut frame = Gray8::new(metrics.width, metrics.height, WHITE);
    let text_scale = (metrics.dpi / 100).clamp(2, 5);
    let coordinate_scale = (metrics.dpi / 100).clamp(2, 4);

    draw_header(&mut frame, state, layout, text_scale);
    draw_board(&mut frame, state, layout, coordinate_scale);
    draw_toolbar(&mut frame, state, layout, text_scale);
    draw_navigation(&mut frame, state, layout, text_scale);
    let analysis_entry = state
        .analysis_available()
        .then_some(layout.toolbar_targets[0].rect);
    let analysis = draw_status(&mut frame, state, layout, text_scale);

    match state.feedback() {
        SolutionFeedback::Wrong => draw_wrong_overlay(&mut frame, layout),
        SolutionFeedback::Complete => draw_complete_overlay(&mut frame, layout),
        SolutionFeedback::None | SolutionFeedback::Correct => {}
    }

    if let Some(pending) = state.pending_promotion() {
        draw_promotion_modal(&mut frame, layout, pending.color(), text_scale);
    }

    if state.collection_picker_open() {
        draw_collection_picker(&mut frame, state, layout, text_scale);
    }

    if state.settings_open() {
        draw_settings_panel(&mut frame, state, layout, text_scale);
    }

    if state.puzzle_goto_open() {
        draw_puzzle_goto(&mut frame, state, layout, text_scale);
    }

    Ok(RenderOutput {
        frame,
        layout,
        damage: vec![layout.viewport],
        analysis,
        analysis_entry,
    })
}

fn draw_header(frame: &mut Gray8, state: &AppState, layout: Layout, control_scale: u32) {
    frame.stroke_rect(layout.header, 3, INK);
    let puzzle = state.active_puzzle();
    let difficulty = puzzle
        .difficulty
        .as_ref()
        .map(|value| format!(" ({value})"))
        .unwrap_or_default();
    let title = format!(
        "{}/{} {}{}",
        state.active_puzzle_index() + 1,
        state.active_collection().puzzles().len(),
        puzzle.id,
        difficulty
    );
    let padding = layout.header.height / 8;
    let side = match puzzle.side_to_move() {
        Color::White => "WHITE TO MOVE",
        Color::Black => "BLACK TO MOVE",
    };
    let side_width = measure_text_bold(side, control_scale) + padding * 2;
    let side_rect = Rect::new(
        layout.header.x + layout.header.width.saturating_sub(side_width) / 2,
        layout.header.y,
        side_width,
        layout.header.height,
    );
    let title_x = layout.refresh.right().saturating_add(padding);
    let title_rect = Rect::new(
        title_x,
        layout.header.y,
        side_rect.x.saturating_sub(title_x + padding),
        layout.header.height,
    );
    let mut title_scale = control_scale;
    while title_scale > 2 && measure_text_bold(&title, title_scale) > title_rect.width {
        title_scale -= 1;
    }
    draw_text_bold_vertically_centered(frame, title_x, title_rect, &title, title_scale, INK);
    draw_text_centered(frame, side_rect, side, control_scale, INK);

    if state.is_current_solved() {
        let badge_width = layout.minimum_touch_px().saturating_mul(2);
        let right = if state.collection_entries().is_empty() {
            layout.settings.x
        } else {
            layout.collection_button.x
        };
        let badge = header_control_visual_rect(
            layout,
            Rect::new(
                right.saturating_sub(badge_width),
                layout.header.y,
                badge_width,
                layout.header.height,
            ),
        );
        draw_button(frame, badge, "SOLVED", true, control_scale);
    }

    draw_settings_button(frame, header_control_visual_rect(layout, layout.settings));
    if !state.collection_entries().is_empty() {
        draw_button(
            frame,
            header_control_visual_rect(layout, layout.collection_button),
            "FILES",
            false,
            control_scale,
        );
    }
    draw_refresh_button(frame, header_control_visual_rect(layout, layout.refresh));
    draw_close_button(frame, header_control_visual_rect(layout, layout.exit));
}

fn header_control_visual_rect(layout: Layout, rect: Rect) -> Rect {
    rect.inset(layout.header.height / 10)
}

fn draw_board(frame: &mut Gray8, state: &AppState, layout: Layout, coordinate_scale: u32) {
    for display_square in 0..64 {
        let rect = layout.square_rect(display_square);
        let row = display_square / 8;
        let column = display_square % 8;
        let tone = if (row + column) % 2 == 0 {
            LIGHT_SQUARE
        } else {
            DARK_SQUARE
        };
        frame.fill_rect(rect, tone);

        let logical_square = if state.flipped() {
            63 - display_square
        } else {
            display_square
        };
        if let Some(piece) = state.board().piece_at(logical_square) {
            draw_piece(frame, piece, rect);
        }
        if state.board().selected() == Some(logical_square) {
            frame.stroke_rect(rect, 6, INK);
            frame.stroke_rect(rect.inset(7), 3, WHITE);
        }
    }
    frame.stroke_rect(layout.board, 4, INK);
    draw_coordinates(frame, state.flipped(), layout, coordinate_scale);
    frame.stroke_rect(layout.board_outer, 2, BOARD_FRAME_GRAY);
}

fn draw_coordinates(frame: &mut Gray8, flipped: bool, layout: Layout, scale: u32) {
    let files = if flipped { "HGFEDCBA" } else { "ABCDEFGH" };
    let ranks = if flipped { "12345678" } else { "87654321" };
    let gutter = layout.coordinate_gutter();
    for (index, file) in files.chars().enumerate() {
        let square = layout.square_rect(index + 56);
        let rect = Rect::new(square.x, layout.board.bottom(), square.width, gutter);
        draw_text_centered(frame, rect, &file.to_string(), scale, INK);
    }
    for (index, rank) in ranks.chars().enumerate() {
        let square = layout.square_rect(index * 8);
        let rect = Rect::new(
            layout.board.x.saturating_sub(gutter),
            square.y,
            gutter,
            square.height,
        );
        draw_text_centered(frame, rect, &rank.to_string(), scale, INK);
    }
}

fn draw_toolbar(frame: &mut Gray8, state: &AppState, layout: Layout, scale: u32) {
    for target in layout.toolbar_targets {
        if target.target == HitTarget::ToggleMode && !state.settings().show_free_mode_button() {
            continue;
        }
        if target.target == HitTarget::ToggleDescription && !state.settings().show_notes_button() {
            continue;
        }
        if target.target == HitTarget::ToggleAnalysis {
            draw_analysis_toggle(frame, layout.control_visual_rect(target.rect), state);
            continue;
        }
        let (label, selected) = match target.target {
            HitTarget::ToggleMode => ("FREE", state.mode() == BoardMode::FreeBoard),
            HitTarget::ToggleDescription => ("NOTE", state.description_visible()),
            HitTarget::ToggleOrientationLock => ("LOCK", state.orientation_locked()),
            HitTarget::Reset => ("RESET", false),
            HitTarget::Flip => ("FLIP", false),
            _ => continue,
        };
        draw_button(
            frame,
            layout.control_visual_rect(target.rect),
            label,
            selected,
            scale,
        );
    }
}

fn draw_analysis_toggle(frame: &mut Gray8, rect: Rect, state: &AppState) {
    let selected = state.analysis_browser_open();
    let background = if selected { INK } else { WHITE };
    let ink = if selected {
        WHITE
    } else if state.analysis_available() {
        INK
    } else {
        DISABLED_INK
    };
    draw_button_chrome(frame, rect, background, ink, selected);
    // An open book: two pages and a central spine, drawn without font glyphs.
    let icon = rect.inset(rect.width.min(rect.height) / 4);
    let thickness = (rect.height / 24).clamp(3, 6);
    frame.stroke_rect(icon, thickness, ink);
    frame.fill_rect(
        Rect::new(icon.x + icon.width / 2, icon.y, thickness, icon.height),
        ink,
    );
    for page in 0..2 {
        for row in 1..=3 {
            frame.fill_rect(
                Rect::new(
                    icon.x + page * icon.width / 2 + thickness * 2,
                    icon.y + row * icon.height / 5,
                    icon.width / 2 - thickness * 4,
                    thickness,
                ),
                ink,
            );
        }
    }
}

fn draw_navigation(frame: &mut Gray8, state: &AppState, layout: Layout, scale: u32) {
    draw_navigation_button(
        frame,
        layout.control_visual_rect(layout.previous),
        "< PREV",
        state.can_previous_puzzle(),
        scale,
    );
    draw_button(
        frame,
        layout.control_visual_rect(layout.goto),
        "GOTO",
        false,
        scale.min(3),
    );
    draw_navigation_button(
        frame,
        layout.control_visual_rect(layout.next),
        "NEXT >",
        state.can_next_puzzle(),
        scale,
    );
}

fn draw_status(
    frame: &mut Gray8,
    state: &AppState,
    layout: Layout,
    scale: u32,
) -> Option<AnalysisPanelOutput> {
    draw_button_chrome(frame, layout.status, WHITE, INK, false);

    let padding = (layout.status.height / 12).max(8);
    let mut content = layout.status.inset(padding);

    if state.analysis_browser_open() {
        return draw_analysis_panel(frame, state, layout, content, scale.min(3));
    }

    if state.feedback() == SolutionFeedback::Correct {
        content.width = content.width.saturating_sub(layout.minimum_touch_px());
    }

    if let Some(message) = state.transient_message() {
        frame.fill_rounded_rect(content, 8, SOFT_GRAY);
        frame.stroke_rect(content, 2, INK);
        draw_wrapped_text(frame, content.inset(8), &format!("! {message}"), scale, INK);
    } else {
        let puzzle = state.active_puzzle();
        if let Some(description) = state
            .description_visible()
            .then_some(puzzle.description.as_deref())
            .flatten()
            .filter(|text| !text.trim().is_empty())
        {
            draw_wrapped_text_with_line_spacing(
                frame,
                content,
                description,
                scale,
                INK,
                DESCRIPTION_EXTRA_LINE_SPACING,
            );
        } else if let Some(topic) = (state.mode() == BoardMode::Solution)
            .then_some(puzzle.topic.as_deref())
            .flatten()
            .filter(|text| !text.trim().is_empty())
        {
            draw_wrapped_text_with_line_spacing(
                frame,
                content,
                topic,
                (scale + 1).min(5),
                INK,
                DESCRIPTION_EXTRA_LINE_SPACING,
            );
        } else {
            let status_line = match state.mode() {
                BoardMode::FreeBoard => "FREE BOARD",
                BoardMode::Solution if state.feedback() == SolutionFeedback::Correct => "CORRECT",
                BoardMode::Solution => "",
            };
            draw_text_bold(frame, content.x, content.y, status_line, scale, INK);
        }
    }

    if state.feedback() == SolutionFeedback::Correct {
        let mark = Rect::new(
            layout
                .status
                .right()
                .saturating_sub(layout.minimum_touch_px()),
            layout.status.y,
            layout.minimum_touch_px(),
            layout.minimum_touch_px(),
        );
        draw_check(frame, mark.inset(mark.width / 5), 5, INK);
    }

    None
}

fn draw_wrong_overlay(frame: &mut Gray8, layout: Layout) {
    let overlay = feedback_rect(layout);
    frame.fill_rect(overlay, WHITE);
    frame.stroke_rect(overlay, 6, INK);
    let inner = overlay.inset(overlay.width / 5);
    draw_thick_line(
        frame,
        i32::try_from(inner.x).unwrap_or(i32::MAX),
        i32::try_from(inner.y).unwrap_or(i32::MAX),
        i32::try_from(inner.right()).unwrap_or(i32::MAX),
        i32::try_from(inner.bottom()).unwrap_or(i32::MAX),
        10,
        INK,
    );
    draw_thick_line(
        frame,
        i32::try_from(inner.right()).unwrap_or(i32::MAX),
        i32::try_from(inner.y).unwrap_or(i32::MAX),
        i32::try_from(inner.x).unwrap_or(i32::MAX),
        i32::try_from(inner.bottom()).unwrap_or(i32::MAX),
        10,
        INK,
    );
}

fn draw_complete_overlay(frame: &mut Gray8, layout: Layout) {
    let overlay = feedback_rect(layout);
    frame.fill_rect(overlay, WHITE);
    frame.stroke_rect(overlay, 6, INK);
    draw_check(frame, overlay.inset(overlay.width / 6), 12, INK);
}

fn feedback_rect(layout: Layout) -> Rect {
    let size = layout.square_size().saturating_mul(2);
    Rect::new(
        layout.board.x + (layout.board.width - size) / 2,
        layout.board.y + (layout.board.height - size) / 2,
        size,
        size,
    )
}

fn draw_promotion_modal(frame: &mut Gray8, layout: Layout, color: Color, scale: u32) {
    frame.fill_rect(layout.promotion_modal, WHITE);
    frame.stroke_rect(layout.promotion_modal, 6, INK);
    let title_height = layout.minimum_touch_px();
    let title = Rect::new(
        layout.promotion_modal.x,
        layout.promotion_modal.y,
        layout.promotion_modal.width,
        title_height,
    );
    draw_text_centered(frame, title, "PROMOTE PAWN", scale, INK);

    const KINDS: [PieceKind; 4] = [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
    ];
    const LABELS: [&str; 4] = ["Q", "R", "B", "N"];
    for (index, rect) in layout.promotion_choices.into_iter().enumerate() {
        frame.fill_rect(rect, SOFT_GRAY);
        frame.stroke_rect(rect, 3, INK);
        let piece_area = Rect::new(rect.x, rect.y, rect.width, rect.height * 3 / 4);
        draw_piece(
            frame,
            chess_core::Piece {
                color,
                kind: KINDS[index],
            },
            piece_area,
        );
        let label = Rect::new(
            rect.x,
            rect.y + rect.height * 3 / 4,
            rect.width,
            rect.height / 4,
        );
        draw_text_centered(frame, label, LABELS[index], scale.min(3), INK);
    }
    draw_button(frame, layout.promotion_cancel, "CANCEL", false, scale);
}

fn draw_puzzle_goto(frame: &mut Gray8, state: &AppState, layout: Layout, scale: u32) {
    frame.fill_rect(layout.goto_modal, WHITE);
    frame.stroke_rect(layout.goto_modal, 6, INK);

    let title = Rect::new(
        layout.goto_modal.x,
        layout.goto_modal.y,
        layout.goto_modal.width,
        layout.minimum_touch_px(),
    );
    draw_text_centered(frame, title, "GO TO PUZZLE", scale, INK);

    draw_button_chrome(frame, layout.goto_input, WHITE, INK, false);
    let input = state.puzzle_goto_input().unwrap_or_default();
    let placeholder;
    let (display, tone) = if input.is_empty() {
        placeholder = format!("1 - {}", state.active_collection().puzzles().len());
        (placeholder.as_str(), DISABLED_INK)
    } else {
        (input, INK)
    };
    draw_text_centered(
        frame,
        layout.goto_input.inset(8),
        display,
        (scale + 1).min(5),
        tone,
    );

    for digit in 0..=9 {
        draw_button(
            frame,
            layout.goto_digits[digit],
            &digit.to_string(),
            false,
            scale,
        );
    }
    draw_button(frame, layout.goto_backspace, "DEL", false, scale.min(3));
    draw_navigation_button(
        frame,
        layout.goto_confirm,
        "CONFIRM",
        state.puzzle_goto_number().is_some(),
        scale.min(3),
    );
    draw_button(frame, layout.goto_cancel, "CANCEL", false, scale.min(3));
}

fn draw_settings_panel(frame: &mut Gray8, state: &AppState, layout: Layout, scale: u32) {
    frame.fill_rect(layout.settings_modal, WHITE);
    frame.stroke_rect(layout.settings_modal, 6, INK);

    let title = Rect::new(
        layout.settings_modal.x,
        layout.settings_modal.y,
        layout.settings_modal.width,
        layout.minimum_touch_px(),
    );
    draw_text_centered(frame, title, "SETTINGS", scale, INK);

    draw_setting_row(
        frame,
        layout.settings_free_mode,
        "FREE MODE BUTTON",
        state.settings().show_free_mode_button(),
        scale.min(3),
    );
    draw_setting_row(
        frame,
        layout.settings_notes,
        "NOTES BUTTON",
        state.settings().show_notes_button(),
        scale.min(3),
    );
    draw_button(frame, layout.settings_close, "CLOSE", false, scale);
}

fn draw_setting_row(frame: &mut Gray8, rect: Rect, label: &str, enabled: bool, scale: u32) {
    draw_button_chrome(frame, rect, WHITE, INK, false);
    let padding = (rect.height / 5).max(8);
    draw_text_bold_vertically_centered(frame, rect.x + padding, rect, label, scale, INK);
    let value = if enabled { "ON" } else { "OFF" };
    let value_width = measure_text_bold(value, scale);
    let value_x = rect
        .right()
        .saturating_sub(padding)
        .saturating_sub(value_width);
    draw_text_bold_vertically_centered(frame, value_x, rect, value, scale, INK);
}

fn draw_collection_picker(frame: &mut Gray8, state: &AppState, layout: Layout, scale: u32) {
    frame.fill_rect(layout.collection_modal, WHITE);
    frame.stroke_rect(layout.collection_modal, 6, INK);

    let title = Rect::new(
        layout.collection_modal.x,
        layout.collection_modal.y,
        layout.collection_modal.width,
        layout.minimum_touch_px(),
    );
    let page_count = state.collection_picker_page_count().max(1);
    draw_text_centered(
        frame,
        title,
        &format!(
            "COLLECTIONS {}/{}",
            state.collection_picker_page() + 1,
            page_count
        ),
        scale,
        INK,
    );

    for (slot, index) in state.collection_picker_visible_range().enumerate() {
        let entry = &state.collection_entries()[index];
        let rect = layout.collection_rows[slot];
        let selected = entry.key() == state.active_collection().key();
        let (background, foreground) = if selected {
            (INK, WHITE)
        } else if entry.error().is_some() {
            (SOFT_GRAY, INK)
        } else {
            (WHITE, INK)
        };
        frame.fill_rect(rect, background);
        frame.stroke_rect(rect, 3, INK);
        let text = match entry.error() {
            Some(error) => format!("{}\n! {error}", entry.label()),
            None => entry.label().to_owned(),
        };
        draw_wrapped_text(frame, rect.inset(10), &text, scale.min(3), foreground);
    }

    draw_navigation_button(
        frame,
        layout.collection_page_previous,
        "< PAGE",
        state.collection_picker_can_previous_page(),
        scale,
    );
    draw_button(frame, layout.collection_close, "CLOSE", false, scale);
    draw_navigation_button(
        frame,
        layout.collection_page_next,
        "PAGE >",
        state.collection_picker_can_next_page(),
        scale,
    );
}

fn draw_navigation_button(frame: &mut Gray8, rect: Rect, label: &str, enabled: bool, scale: u32) {
    if enabled {
        draw_button(frame, rect, label, false, scale);
    } else {
        draw_button_chrome(frame, rect, SOFT_GRAY, DISABLED_INK, false);
        draw_text_centered(frame, rect.inset(6), label, scale, DISABLED_INK);
    }
}

fn draw_refresh_button(frame: &mut Gray8, rect: Rect) {
    draw_button_chrome(frame, rect, WHITE, INK, false);
    let size = rect.width.min(rect.height);
    let icon = rect.inset(size / 4);
    let thickness = (size / 18).clamp(4, 8);
    draw_refresh_icon(frame, icon, thickness, INK);
}

fn draw_refresh_icon(frame: &mut Gray8, rect: Rect, thickness: u32, tone: u8) {
    let left = rect.x.saturating_add(rect.width / 5);
    let right = rect
        .right()
        .saturating_sub(1)
        .saturating_sub(rect.width / 5);
    let top = rect.y.saturating_add(rect.height / 4);
    let bottom = rect
        .bottom()
        .saturating_sub(1)
        .saturating_sub(rect.height / 4);
    let mid_y = rect.y.saturating_add(rect.height / 2);
    let arrow = (rect.width / 5).max(2);

    draw_thick_line(
        frame,
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(top).unwrap_or(i32::MAX),
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(top).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(top).unwrap_or(i32::MAX),
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(mid_y).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(top).unwrap_or(i32::MAX),
        i32::try_from(right.saturating_sub(arrow)).unwrap_or(i32::MAX),
        i32::try_from(top.saturating_sub(arrow / 2)).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(top).unwrap_or(i32::MAX),
        i32::try_from(right.saturating_sub(arrow)).unwrap_or(i32::MAX),
        i32::try_from(top.saturating_add(arrow / 2)).unwrap_or(i32::MAX),
        thickness,
        tone,
    );

    draw_thick_line(
        frame,
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(mid_y).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        i32::try_from(left.saturating_add(arrow)).unwrap_or(i32::MAX),
        i32::try_from(bottom.saturating_sub(arrow / 2)).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(left).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        i32::try_from(left.saturating_add(arrow)).unwrap_or(i32::MAX),
        i32::try_from(bottom.saturating_add(arrow / 2)).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
}

fn draw_settings_button(frame: &mut Gray8, rect: Rect) {
    draw_button_chrome(frame, rect, WHITE, INK, false);
    let icon = rect.inset(rect.width.min(rect.height) / 4);
    let thickness = (rect.height / 24).clamp(3, 6);
    for row in 0..3 {
        let y = icon.y + (row + 1) * icon.height / 4;
        frame.fill_rect(Rect::new(icon.x, y, icon.width, thickness), INK);
        let knob_x = if row == 1 {
            icon.x + icon.width / 3
        } else {
            icon.x + icon.width * 2 / 3
        };
        frame.fill_rect(
            Rect::new(
                knob_x.saturating_sub(thickness),
                y.saturating_sub(thickness),
                thickness * 3,
                thickness * 3,
            ),
            INK,
        );
    }
}

fn draw_close_button(frame: &mut Gray8, rect: Rect) {
    draw_button_chrome(frame, rect, WHITE, INK, false);
    let size = rect.width.min(rect.height);
    let icon = rect.inset(size / 4);
    let thickness = (size / 18).clamp(4, 8);
    draw_close_icon(frame, icon, thickness, INK);
}

fn draw_close_icon(frame: &mut Gray8, rect: Rect, thickness: u32, tone: u8) {
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);
    draw_thick_line(
        frame,
        i32::try_from(rect.x).unwrap_or(i32::MAX),
        i32::try_from(rect.y).unwrap_or(i32::MAX),
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(right).unwrap_or(i32::MAX),
        i32::try_from(rect.y).unwrap_or(i32::MAX),
        i32::try_from(rect.x).unwrap_or(i32::MAX),
        i32::try_from(bottom).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
}

fn draw_button(frame: &mut Gray8, rect: Rect, label: &str, selected: bool, scale: u32) {
    let (background, foreground) = if selected { (INK, WHITE) } else { (WHITE, INK) };
    draw_button_chrome(frame, rect, background, INK, selected);
    draw_text_centered(frame, rect.inset(6), label, scale, foreground);
}

fn draw_button_chrome(frame: &mut Gray8, rect: Rect, background: u8, border: u8, selected: bool) {
    let radius = (rect.height / 6).clamp(6, 18);
    frame.fill_rounded_rect(rect, radius, border);

    let inner = rect.inset(3);
    frame.fill_rounded_rect(inner, radius.saturating_sub(3), background);

    if selected {
        let accent = rect.inset(6);
        frame.fill_rounded_rect(accent, radius.saturating_sub(6), WHITE);
        let accent_inner = rect.inset(8);
        frame.fill_rounded_rect(accent_inner, radius.saturating_sub(8), background);
    }
}

fn draw_check(frame: &mut Gray8, rect: Rect, thickness: u32, tone: u8) {
    let x0 = rect.x + rect.width / 8;
    let y0 = rect.y + rect.height / 2;
    let x1 = rect.x + rect.width * 2 / 5;
    let y1 = rect.y + rect.height * 3 / 4;
    let x2 = rect.x + rect.width * 7 / 8;
    let y2 = rect.y + rect.height / 4;
    draw_thick_line(
        frame,
        i32::try_from(x0).unwrap_or(i32::MAX),
        i32::try_from(y0).unwrap_or(i32::MAX),
        i32::try_from(x1).unwrap_or(i32::MAX),
        i32::try_from(y1).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
    draw_thick_line(
        frame,
        i32::try_from(x1).unwrap_or(i32::MAX),
        i32::try_from(y1).unwrap_or(i32::MAX),
        i32::try_from(x2).unwrap_or(i32::MAX),
        i32::try_from(y2).unwrap_or(i32::MAX),
        thickness,
        tone,
    );
}

fn draw_thick_line(
    frame: &mut Gray8,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: u32,
    tone: u8,
) {
    let half = i32::try_from(thickness / 2).unwrap_or(i32::MAX);
    for offset_y in -half..=half {
        for offset_x in -half..=half {
            frame.draw_line(
                x0.saturating_add(offset_x),
                y0.saturating_add(offset_y),
                x1.saturating_add(offset_x),
                y1.saturating_add(offset_y),
                tone,
            );
        }
    }
}

/// Draw a transient sleep notice over the existing board without changing app state.
pub fn draw_sleeping_overlay(frame: &mut Gray8, layout: Layout, metrics: DisplayMetrics) -> Rect {
    let board = layout.board;
    let width = board.width * 2 / 3;
    let height = board.height / 7;
    let rect = Rect::new(
        board.x + (board.width - width) / 2,
        board.y + (board.height - height) / 2,
        width,
        height,
    );
    frame.fill_rect(rect, WHITE);
    frame.stroke_rect(rect, (metrics.dpi / 100).max(2), INK);
    draw_text_centered(
        frame,
        rect.inset(12),
        "Sleeping...",
        (metrics.dpi / 60).clamp(2, 8),
        INK,
    );
    rect
}
