use chess_core::Board;
use chess_render::{Layout, Rect};
const MAX_TARGETED_CLEAN_SQUARES: usize = 4;
/// Compare visible piece occupancy only. Selection, feedback, and modal changes
/// remain ordinary partial damage. Large board changes and orientation changes
/// use one clean board update instead of many per-square flashes.
pub fn clean_regions_for_board_change(
    before: &Board,
    after: &Board,
    before_flipped: bool,
    after_flipped: bool,
    layout: Layout,
) -> Vec<Rect> {
    let changed = (0..64)
        .filter(|&square| before.piece_at(square) != after.piece_at(square))
        .collect::<Vec<_>>();

    if changed.is_empty() {
        return Vec::new();
    }
    if before_flipped != after_flipped || changed.len() > MAX_TARGETED_CLEAN_SQUARES {
        return vec![layout.board];
    }

    changed
        .into_iter()
        .map(|logical| {
            let display = if after_flipped { 63 - logical } else { logical };
            layout.square_rect(display)
        })
        .collect()
}
