//! Platform-neutral deterministic grayscale renderer for Kindle Chess Puzzles.

#![forbid(unsafe_code)]

mod analysis_panel;
mod button;
mod canvas;
mod damage;
mod font;
mod geometry;
mod layout;
mod pieces;
mod render;
mod screen;
pub use screen::ScreenState;

pub use analysis_panel::{AnalysisMoveChip, AnalysisMoveChipSource, AnalysisPanelOutput};
pub use button::{ButtonIcon, ButtonSpec, ButtonType};
pub use canvas::{CanvasError, Gray8};
pub use damage::{calculate_damage, compact_damage, merge_clipped};
pub use geometry::Rect;
pub use layout::{
    ControlTarget, DisplayMetrics, HitTarget, Layout, LayoutError, ToolbarAlignment,
    APP_TOOLBAR_ALIGNMENT, MIN_TOUCH_MM,
};
pub use render::{draw_sleeping_overlay, render, RenderOutput};

pub fn render_puzzles(
    state: &chess_core::puzzle_app::PuzzleState,
    metrics: DisplayMetrics,
) -> Result<RenderOutput, LayoutError> {
    render(state, metrics)
}
pub fn render_review(
    state: &chess_core::review_app::ReviewApp,
    metrics: DisplayMetrics,
) -> Result<RenderOutput, LayoutError> {
    render(state, metrics)
}
