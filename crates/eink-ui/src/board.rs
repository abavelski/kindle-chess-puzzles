use crate::{font::draw_text_centered, pieces::draw_piece, Gray8, Rect};
const LIGHT_SQUARE: u8 = 238;
const DARK_SQUARE: u8 = 184;
const INK: u8 = 0;
const WHITE: u8 = 255;
const BOARD_FRAME_GRAY: u8 = 160;
#[derive(Clone, Copy, Debug)]
pub struct BoardGeometry {
    pub board: Rect,
    pub board_outer: Rect,
    pub gutter: u32,
}
impl BoardGeometry {
    pub fn square_rect(self, display_square: usize) -> Rect {
        let size = self.board.width / 8;
        Rect::new(
            self.board.x + (display_square as u32 % 8) * size,
            self.board.y + (display_square as u32 / 8) * size,
            size,
            size,
        )
    }
    pub fn coordinate_gutter(self) -> u32 {
        self.gutter
    }
}
/// Convert between canonical and display squares; the transform is its own inverse.
pub fn oriented_square(square: usize, flipped: bool) -> usize {
    debug_assert!(square < 64);
    if flipped {
        63 - square
    } else {
        square
    }
}

pub fn draw_board(
    frame: &mut Gray8,
    board: &chess_model::Board,
    flipped: bool,
    layout: BoardGeometry,
    coordinate_scale: u32,
) {
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

        let logical_square = oriented_square(display_square, flipped);
        if let Some(piece) = board.piece_at(logical_square) {
            draw_piece(frame, piece, rect);
        }
        if board.selected() == Some(logical_square) {
            frame.stroke_rect(rect, 6, INK);
            frame.stroke_rect(rect.inset(7), 3, WHITE);
        }
    }
    frame.stroke_rect(layout.board, 4, INK);
    draw_coordinates(frame, flipped, layout, coordinate_scale);
    frame.stroke_rect(layout.board_outer, 2, BOARD_FRAME_GRAY);
}

fn draw_coordinates(frame: &mut Gray8, flipped: bool, layout: BoardGeometry, scale: u32) {
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
