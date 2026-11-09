//! Product-neutral grayscale canvas, chess board, text and analysis widgets.
#![forbid(unsafe_code)]
pub mod analysis_panel;
pub mod board;
pub mod button;
pub mod canvas;
pub mod damage;
pub mod font;
pub mod geometry;
pub mod pieces;
pub use canvas::{CanvasError, Gray8};
pub use geometry::Rect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayMetrics {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
}

pub mod move_list;
