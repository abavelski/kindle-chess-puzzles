//! Measured waveform capabilities are opt-in. AUTO is the baseline until the
//! Task 07 Scribe checkpoint verifies explicit waveforms and a cleaning cadence.
use chess_render::{Gray8, Rect};
pub use fbink_sys::RefreshMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentClass {
    Monochrome,
    Grayscale,
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
