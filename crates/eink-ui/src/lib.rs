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
