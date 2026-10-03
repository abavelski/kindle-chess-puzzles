//! Platform-neutral deterministic grayscale renderer for Kindle Chess Puzzles.

#![forbid(unsafe_code)]

mod canvas;
mod damage;
mod font;
mod geometry;
mod layout;
mod pieces;
mod render;

pub use canvas::{CanvasError, Gray8};
pub use damage::{calculate_damage, compact_damage, merge_clipped};
pub use geometry::Rect;
pub use layout::{ControlTarget, DisplayMetrics, HitTarget, Layout, LayoutError, MIN_TOUCH_MM};
pub use render::{render, RenderOutput};
