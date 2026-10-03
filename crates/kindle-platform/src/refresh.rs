//! Measured waveform capabilities are opt-in. AUTO is the baseline until the
//! Task 07 Scribe checkpoint verifies explicit waveforms and a cleaning cadence.
use chess_core::Board;
use chess_render::{merge_clipped, Gray8, Layout, Rect};
pub use fbink_sys::RefreshMode;

const MAX_TARGETED_CLEAN_SQUARES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentClass {
    Monochrome,
    Grayscale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshStrength {
    Partial,
    Clean,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentRegion {
    pub rect: Rect,
    pub strength: RefreshStrength,
}

impl PresentRegion {
    pub const fn partial(rect: Rect) -> Self {
        Self {
            rect,
            strength: RefreshStrength::Partial,
        }
    }

    pub const fn clean(rect: Rect) -> Self {
        Self {
            rect,
            strength: RefreshStrength::Clean,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RefreshPolicy {
    pub fast_mono_verified: bool,
    pub gray_verified: bool,
    pub full_every: Option<u32>,
    updates_since_full: u32,
}

impl RefreshPolicy {
    pub fn mode(&self, content: ContentClass, full: bool) -> RefreshMode {
        if full {
            return RefreshMode::Full;
        }
        match content {
            ContentClass::Monochrome if self.fast_mono_verified => RefreshMode::FastMono,
            ContentClass::Grayscale if self.gray_verified => RefreshMode::GrayPartial,
            _ => RefreshMode::AutoPartial,
        }
    }
    pub fn full_due(&self) -> bool {
        self.full_every
            .is_some_and(|n| self.updates_since_full.saturating_add(1) >= n.max(1))
    }
    pub fn completed(&mut self, full: bool) {
        self.updates_since_full = if full {
            0
        } else {
            self.updates_since_full.saturating_add(1)
        };
    }
}

pub fn classify_region(previous: Option<&Gray8>, current: &Gray8, rect: Rect) -> ContentClass {
    for frame in previous.into_iter().chain(std::iter::once(current)) {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                if !matches!(frame.pixel(x, y), Some(0 | 255)) {
                    return ContentClass::Grayscale;
                }
            }
        }
    }
    ContentClass::Monochrome
}

pub fn pack_region(frame: &Gray8, rect: Rect) -> Result<Vec<u8>, fbink_sys::ValidationError> {
    fbink_sys::validate_rect(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        frame.width(),
        frame.height(),
    )?;
    let mut pixels = Vec::with_capacity(rect.width as usize * rect.height as usize);
    for y in rect.y..rect.bottom() {
        let start = (y * frame.stride() + rect.x) as usize;
        pixels.extend_from_slice(&frame.pixels()[start..start + rect.width as usize]);
    }
    Ok(pixels)
}

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

/// Replace ordinary damage covered by clean regions while preserving damage
/// outside them. Partial submissions are emitted first and clean submissions
/// last so changed board cells finish with the stronger waveform.
pub fn plan_present_regions(
    damage: &[Rect],
    clean_regions: &[Rect],
    viewport: Rect,
) -> Vec<PresentRegion> {
    let clean = normalize_clean_regions(clean_regions, viewport);
    let mut partial = merge_clipped(damage, viewport);

    for clean_rect in &clean {
        partial = partial
            .into_iter()
            .flat_map(|rect| subtract_rect(rect, *clean_rect))
            .collect();
    }
    partial = merge_clipped(&partial, viewport);

    let mut planned = partial
        .into_iter()
        .map(PresentRegion::partial)
        .collect::<Vec<_>>();
    planned.extend(clean.into_iter().map(PresentRegion::clean));
    planned
}

fn normalize_clean_regions(regions: &[Rect], viewport: Rect) -> Vec<Rect> {
    let mut result = Vec::new();
    for rect in regions {
        let Some(mut pending) = clip_rect(*rect, viewport) else {
            continue;
        };
        let mut index = 0;
        while index < result.len() {
            let other = result[index];
            if pending.intersects(other) || pending == other {
                pending = union_rect(pending, other);
                result.remove(index);
                index = 0;
            } else {
                index += 1;
            }
        }
        result.push(pending);
    }
    result.sort_by_key(|rect| (rect.y, rect.x));
    result
}

fn clip_rect(rect: Rect, viewport: Rect) -> Option<Rect> {
    let x = rect.x.max(viewport.x);
    let y = rect.y.max(viewport.y);
    let right = rect.right().min(viewport.right());
    let bottom = rect.bottom().min(viewport.bottom());
    (right > x && bottom > y).then(|| Rect::new(x, y, right - x, bottom - y))
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect::new(
        x,
        y,
        a.right().max(b.right()) - x,
        a.bottom().max(b.bottom()) - y,
    )
}

fn subtract_rect(rect: Rect, cut: Rect) -> Vec<Rect> {
    if !rect.intersects(cut) {
        return vec![rect];
    }

    let ix = rect.x.max(cut.x);
    let iy = rect.y.max(cut.y);
    let ir = rect.right().min(cut.right());
    let ib = rect.bottom().min(cut.bottom());
    let mut result = Vec::with_capacity(4);

    if rect.y < iy {
        result.push(Rect::new(rect.x, rect.y, rect.width, iy - rect.y));
    }
    if ib < rect.bottom() {
        result.push(Rect::new(rect.x, ib, rect.width, rect.bottom() - ib));
    }
    if rect.x < ix {
        result.push(Rect::new(rect.x, iy, ix - rect.x, ib - iy));
    }
    if ir < rect.right() {
        result.push(Rect::new(ir, iy, rect.right() - ir, ib - iy));
    }
    result
}
