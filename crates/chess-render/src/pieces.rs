//! Deterministic rasterization of generated Sashité Western vector layers.

use crate::{Gray8, Rect};
use chess_core::Piece;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Cmd {
    Move(i32, i32),
    Line(i32, i32),
    Quad(i32, i32, i32, i32),
    Close,
}

#[path = "pieces_generated.rs"]
mod generated;

pub(crate) fn draw_piece(canvas: &mut Gray8, piece: Piece, rect: Rect) {
    let inset = (rect.width.min(rect.height) / 16).max(2);
    let target = rect.inset(inset);
    if target.width == 0 || target.height == 0 {
        return;
    }
    let size = target.width.min(target.height);
    let x = target.x + (target.width - size) / 2;
    let y = target.y + (target.height - size) / 2;
    let square = Rect::new(x, y, size, size);

    for layer in generated::layers(piece.color, piece.kind) {
        draw_layer(canvas, square, layer.commands, layer.tone);
    }
}

fn draw_layer(canvas: &mut Gray8, rect: Rect, commands: &[Cmd], tone: u8) {
    const QUAD_STEPS: i64 = 16;
    const QUAD_DENOMINATOR: i64 = QUAD_STEPS * QUAD_STEPS;

    let mut polygon = Vec::<(i32, i32)>::new();
    let mut current = None::<(i32, i32)>;

    for command in commands {
        match *command {
            Cmd::Move(x, y) => {
                fill_open_polygon(canvas, &mut polygon, tone);
                let point = transform(rect, x, y);
                polygon.push(point);
                current = Some(point);
            }
            Cmd::Line(x, y) => {
                let point = transform(rect, x, y);
                polygon.push(point);
                current = Some(point);
            }
            Cmd::Quad(cx, cy, x, y) => {
                let Some(start) = current else {
                    continue;
                };
                let control = transform(rect, cx, cy);
                let end = transform(rect, x, y);
                for step in 1..=QUAD_STEPS {
                    let inverse = QUAD_STEPS - step;
                    let px = (inverse * inverse * i64::from(start.0)
                        + 2 * inverse * step * i64::from(control.0)
                        + step * step * i64::from(end.0)
                        + QUAD_DENOMINATOR / 2)
                        / QUAD_DENOMINATOR;
                    let py = (inverse * inverse * i64::from(start.1)
                        + 2 * inverse * step * i64::from(control.1)
                        + step * step * i64::from(end.1)
                        + QUAD_DENOMINATOR / 2)
                        / QUAD_DENOMINATOR;
                    polygon.push((
                        i32::try_from(px).unwrap_or(if px < 0 { i32::MIN } else { i32::MAX }),
                        i32::try_from(py).unwrap_or(if py < 0 { i32::MIN } else { i32::MAX }),
                    ));
                }
                current = Some(end);
            }
            Cmd::Close => {
                fill_open_polygon(canvas, &mut polygon, tone);
                current = None;
            }
        }
    }
    fill_open_polygon(canvas, &mut polygon, tone);
}

fn fill_open_polygon(canvas: &mut Gray8, polygon: &mut Vec<(i32, i32)>, tone: u8) {
    if polygon.len() >= 3 {
        canvas.fill_polygon(polygon, tone);
    }
    polygon.clear();
}

fn transform(rect: Rect, x: i32, y: i32) -> (i32, i32) {
    let width = i64::from(rect.width);
    let height = i64::from(rect.height);
    let px = i64::from(rect.x) + (i64::from(x) * width + 500) / 1000;
    let py = i64::from(rect.y) + (i64::from(y) * height + 500) / 1000;
    (
        i32::try_from(px).unwrap_or(i32::MAX),
        i32::try_from(py).unwrap_or(i32::MAX),
    )
}

#[cfg(test)]
mod tests {
    use super::draw_piece;
    use crate::{Gray8, Rect};
    use chess_core::{Color, Piece, PieceKind};

    #[test]
    fn all_twelve_generated_piece_assets_rasterize() {
        for color in [Color::White, Color::Black] {
            for kind in [
                PieceKind::Pawn,
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen,
                PieceKind::King,
            ] {
                let mut canvas = Gray8::new(128, 128, 230);
                let before = canvas.checksum64();
                draw_piece(
                    &mut canvas,
                    Piece { color, kind },
                    Rect::new(0, 0, 128, 128),
                );
                assert_ne!(canvas.checksum64(), before, "{color:?} {kind:?}");
            }
        }
    }
}
