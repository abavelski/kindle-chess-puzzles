//! Pixel-verified damage in stable 64px tiles. Rendering remains independent
//! of display waveforms. Comparing visible pixels also catches text overflow.
use crate::{Gray8, Rect};

pub fn calculate_damage(previous: Option<&Gray8>, current: &Gray8) -> Vec<Rect> {
    let viewport = Rect::new(0, 0, current.width(), current.height());
    let Some(previous) =
        previous.filter(|old| old.width() == current.width() && old.height() == current.height())
    else {
        return vec![viewport];
    };
    let mut regions = Vec::new();
    for y in (0..current.height()).step_by(64) {
        for x in (0..current.width()).step_by(64) {
            let rect = Rect::new(
                x,
                y,
                64.min(current.width() - x),
                64.min(current.height() - y),
            );
            let changed = (y..rect.bottom()).any(|row| {
                let old_start = (row * previous.stride() + x) as usize;
                let new_start = (row * current.stride() + x) as usize;
                previous.pixels()[old_start..old_start + rect.width as usize]
                    != current.pixels()[new_start..new_start + rect.width as usize]
            });
            if changed {
                regions.push(rect);
            }
        }
    }
    merge_clipped(&regions, viewport)
}

/// Clip first, then merge overlaps and aligned touching rectangles. Revisit
/// earlier rectangles after each merge so transitive overlaps are covered.
pub fn merge_clipped(regions: &[Rect], viewport: Rect) -> Vec<Rect> {
    let mut result: Vec<Rect> = Vec::new();
    for rect in regions {
        let x = rect.x.max(viewport.x);
        let y = rect.y.max(viewport.y);
        let right = rect.right().min(viewport.right());
        let bottom = rect.bottom().min(viewport.bottom());
        if right <= x || bottom <= y {
            continue;
        }
        let mut pending = Rect::new(x, y, right - x, bottom - y);
        let mut index = 0;
        while index < result.len() {
            let other = result[index];
            let horizontal = pending.y == other.y
                && pending.height == other.height
                && pending.x <= other.right()
                && other.x <= pending.right();
            let vertical = pending.x == other.x
                && pending.width == other.width
                && pending.y <= other.bottom()
                && other.y <= pending.bottom();
            if pending.intersects(other) || horizontal || vertical {
                let x = pending.x.min(other.x);
                let y = pending.y.min(other.y);
                pending = Rect::new(
                    x,
                    y,
                    pending.right().max(other.right()) - x,
                    pending.bottom().max(other.bottom()) - y,
                );
                result.remove(index);
                index = 0;
            } else {
                index += 1;
            }
        }
        result.push(pending);
    }
    result.sort_by_key(|r| (r.y, r.x));
    result
}

/// Bound expensive e-ink submissions within each renderer region. Small,
/// separated piece changes stay separate; fragmented board/orientation changes
/// use one board bounding box, without merging unrelated header/status regions.
pub fn compact_damage(regions: &[Rect], layout: crate::Layout) -> Vec<Rect> {
    let clipped = merge_clipped(regions, layout.viewport);
    if clipped.contains(&layout.viewport) {
        return vec![layout.viewport];
    }
    let areas = [
        layout.board_outer,
        layout.header,
        layout.toolbar,
        layout.previous,
        layout.next,
        layout.status,
    ];
    let mut groups = vec![Vec::new(); areas.len() + 1];
    for rect in clipped {
        let group = areas
            .iter()
            .position(|area| area.intersects(rect))
            .unwrap_or(areas.len());
        groups[group].push(rect);
    }
    let mut result = Vec::new();
    for (index, mut group) in groups.into_iter().enumerate() {
        if group.is_empty() {
            continue;
        }
        if index < areas.len() {
            let bounding = bounds(&group);
            let area_sum: u64 = group
                .iter()
                .map(|r| u64::from(r.width) * u64::from(r.height))
                .sum();
            if group.len() > 4
                || u64::from(bounding.width) * u64::from(bounding.height)
                    <= area_sum.saturating_mul(2)
            {
                group = vec![bounding];
            }
        }
        result.extend(group);
    }
    result.sort_by_key(|rect| (rect.y, rect.x));
    result
}

fn bounds(regions: &[Rect]) -> Rect {
    let x = regions.iter().map(|r| r.x).min().unwrap_or(0);
    let y = regions.iter().map(|r| r.y).min().unwrap_or(0);
    let right = regions.iter().map(|r| r.right()).max().unwrap_or(x);
    let bottom = regions.iter().map(|r| r.bottom()).max().unwrap_or(y);
    Rect::new(x, y, right - x, bottom - y)
}
