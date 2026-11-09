//! Product-neutral move-chip chrome and deterministic pagination shared by analysis and live games.
use crate::{font::draw_text_centered, Gray8, Rect};
const WHITE: u8 = 255;
const INK: u8 = 0;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageWindow {
    pub page: usize,
    pub page_count: usize,
    pub range: std::ops::Range<usize>,
}
impl PageWindow {
    pub fn new(count: usize, capacity: usize, requested: usize) -> Self {
        let capacity = capacity.max(1);
        let page_count = count.div_ceil(capacity).max(1);
        let page = requested.min(page_count - 1);
        let start = page.saturating_mul(capacity).min(count);
        Self {
            page,
            page_count,
            range: start..start.saturating_add(capacity).min(count),
        }
    }
}
pub fn draw_move_chip(frame: &mut Gray8, rect: Rect, label: &str, selected: bool, scale: u32) {
    let (background, foreground) = if selected { (INK, WHITE) } else { (WHITE, INK) };
    frame.fill_rect(rect, background);
    draw_text_centered(frame, rect.inset(4), label, scale, foreground);
}

pub fn draw_page_control(frame: &mut Gray8, rect: Rect, label: &str, scale: u32) {
    frame.fill_rect(rect, WHITE);
    frame.stroke_rect(rect, 2.min(rect.width).min(rect.height), INK);
    draw_text_centered(frame, rect.inset(4), label, scale, INK);
}
