//! Platform-neutral deterministic grayscale renderer for Kindle Chess Puzzles.

#![forbid(unsafe_code)]

mod analysis_panel;
mod canvas;
mod damage;
mod font;
mod geometry;
mod layout;
mod pieces;
mod render;

pub use analysis_panel::{AnalysisMoveChip, AnalysisMoveChipSource, AnalysisPanelOutput};
pub use canvas::{CanvasError, Gray8};
pub use damage::{calculate_damage, compact_damage, merge_clipped};
pub use geometry::Rect;
pub use layout::{
    ControlTarget, DisplayMetrics, HitTarget, Layout, LayoutError, ToolbarAlignment,
    APP_TOOLBAR_ALIGNMENT, MIN_TOUCH_MM,
};
pub use render::{draw_sleeping_overlay, render, RenderOutput};
