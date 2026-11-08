pub use eink_ui::damage::{calculate_damage, merge_clipped};
pub fn compact_damage(regions: &[crate::Rect], layout: crate::Layout) -> Vec<crate::Rect> {
    eink_ui::damage::compact_damage(
        regions,
        layout.viewport,
        &[
            layout.board_outer,
            layout.header,
            layout.toolbar,
            layout.previous,
            layout.next,
            layout.status,
        ],
    )
}
