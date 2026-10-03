//! Safe FBInk-backed display adapter for the measured Kindle Scribe target.

use crate::{
    classify_region, pack_region, plan_present_regions, PresentRegion, RefreshPolicy,
    RefreshStrength,
};
use chess_render::{Gray8, Rect};
use fbink_sys::{FbInk, FbInkError, FbInkState};
use std::fmt;
use std::time::{Duration, Instant};

pub const SCRIBE_DPI: u32 = 300;
pub const SCRIBE_WIDTH: u32 = 1860;
pub const SCRIBE_HEIGHT: u32 = 2480;
pub const SCRIBE_ROTATION: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayState {
    pub width: u32,
    pub height: u32,
    pub scanline_stride: u32,
    pub bpp: u32,
    pub rotation: u8,
    pub is_y8: bool,
}

impl From<FbInkState> for DisplayState {
    fn from(value: FbInkState) -> Self {
        Self {
            width: value.width,
            height: value.height,
            scanline_stride: value.scanline_stride,
            bpp: value.bpp,
            rotation: value.rotation,
            is_y8: value.is_y8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentError {
    GeometryMismatch,
    UnsupportedPixelFormat,
    NonTightFrame,
}

impl fmt::Display for PresentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::GeometryMismatch => "rendered frame dimensions do not match the FBInk display",
            Self::UnsupportedPixelFormat => {
                "Task 04 requires the measured 8-bit Y8 framebuffer mode"
            }
            Self::NonTightFrame => {
                "Task 04 FBInk presentation requires a tightly packed Gray8 frame"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PresentError {}

#[derive(Debug)]
pub enum DisplayError {
    FbInk(FbInkError),
    Present(PresentError),
    UnsupportedScribeState(DisplayState),
}

impl fmt::Display for DisplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FbInk(error) => write!(formatter, "FBInk: {error}"),
            Self::Present(error) => write!(formatter, "display validation: {error}"),
            Self::UnsupportedScribeState(state) => write!(
                formatter,
                "unsupported Scribe display state: {}x{}, stride {}, {}bpp, rotation {}, y8={}",
                state.width,
                state.height,
                state.scanline_stride,
                state.bpp,
                state.rotation,
                state.is_y8
            ),
        }
    }
}

impl std::error::Error for DisplayError {}

impl From<FbInkError> for DisplayError {
    fn from(value: FbInkError) -> Self {
        Self::FbInk(value)
    }
}

impl From<PresentError> for DisplayError {
    fn from(value: PresentError) -> Self {
        Self::Present(value)
    }
}

pub fn validate_presentable_frame(state: &DisplayState, frame: &Gray8) -> Result<(), PresentError> {
    if state.bpp != 8 || !state.is_y8 {
        return Err(PresentError::UnsupportedPixelFormat);
    }
    if state.width != frame.width() || state.height != frame.height() {
        return Err(PresentError::GeometryMismatch);
    }
    if frame.stride() != frame.width() {
        return Err(PresentError::NonTightFrame);
    }
    Ok(())
}

pub fn validate_task04_scribe_state(state: DisplayState) -> Result<(), DisplayError> {
    if state.width != SCRIBE_WIDTH
        || state.height != SCRIBE_HEIGHT
        || state.bpp != 8
        || !state.is_y8
        || state.rotation != SCRIBE_ROTATION
    {
        return Err(DisplayError::UnsupportedScribeState(state));
    }
    Ok(())
}

pub fn validate_damage(frame: &Gray8, regions: &[Rect]) -> Result<(), FbInkError> {
    for rect in regions {
        fbink_sys::validate_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            frame.width(),
            frame.height(),
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PresentTiming {
    pub submit: Duration,
    pub complete: Duration,
    pub regions: usize,
    pub full: bool,
}

/// Host-testable submission boundary; the concrete adapter owns FBInk state.
pub trait RegionPresenter {
    fn submit(
        &mut self,
        rect: Rect,
        pixels: &[u8],
        mode: fbink_sys::RefreshMode,
    ) -> Result<(), FbInkError>;
    fn wait(&mut self) -> Result<(), FbInkError>;
}

impl RegionPresenter for FbInk {
    fn submit(
        &mut self,
        rect: Rect,
        pixels: &[u8],
        mode: fbink_sys::RefreshMode,
    ) -> Result<(), FbInkError> {
        self.present_region(rect.x, rect.y, rect.width, rect.height, pixels, mode)
    }
    fn wait(&mut self) -> Result<(), FbInkError> {
        self.wait_for_complete()
    }
}

/// Submit a complete dirty batch before waiting on its last update marker.
/// Pixel writes remain regional; the caller advances history only on success.
pub fn submit_regions<P: RegionPresenter>(
    backend: &mut P,
    frame: &Gray8,
    previous: Option<&Gray8>,
    regions: &[PresentRegion],
    policy: &RefreshPolicy,
    full: bool,
) -> Result<PresentTiming, FbInkError> {
    for region in regions {
        validate_damage(frame, std::slice::from_ref(&region.rect))?;
    }
    let start = Instant::now();
    for region in regions {
        let pixels = pack_region(frame, region.rect)?;
        let mode = if full {
            fbink_sys::RefreshMode::Full
        } else {
            match region.strength {
                RefreshStrength::Partial => {
                    policy.mode(classify_region(previous, frame, region.rect), false)
                }
                RefreshStrength::Clean => fbink_sys::RefreshMode::Clean,
            }
        };
        backend.submit(region.rect, &pixels, mode)?;
    }
    let submit = start.elapsed();
    if !regions.is_empty() {
        backend.wait()?;
    }
    Ok(PresentTiming {
        submit,
        complete: start.elapsed(),
        regions: regions.len(),
        full,
    })
}
pub struct KindleDisplay {
    fbink: FbInk,
    state: DisplayState,
}

impl KindleDisplay {
    pub fn open() -> Result<Self, DisplayError> {
        let fbink = FbInk::open()?;
        let state = DisplayState::from(fbink.state()?);
        validate_task04_scribe_state(state)?;
        Ok(Self { fbink, state })
    }

    pub const fn state(&self) -> DisplayState {
        self.state
    }

    pub fn fbink_version(&self) -> Result<String, DisplayError> {
        Ok(self.fbink.version()?)
    }

    pub fn reinitialize(&mut self) -> Result<DisplayState, DisplayError> {
        self.fbink.reinitialize()?;
        self.state = DisplayState::from(self.fbink.state()?);
        validate_task04_scribe_state(self.state)?;
        Ok(self.state)
    }

    pub fn present(&mut self, frame: &Gray8) -> Result<(), DisplayError> {
        self.present_damage(
            frame,
            None,
            &[Rect::new(0, 0, frame.width(), frame.height())],
            &[],
            &mut RefreshPolicy::default(),
            true,
        )?;
        Ok(())
    }

    /// The caller commits its previous frame only after this returns success.
    /// Validate the whole batch first; a later failure must never advance damage history.
    pub fn present_damage(
        &mut self,
        frame: &Gray8,
        previous: Option<&Gray8>,
        regions: &[Rect],
        clean_regions: &[Rect],
        policy: &mut RefreshPolicy,
        force_full: bool,
    ) -> Result<PresentTiming, DisplayError> {
        self.reinitialize()?;
        validate_presentable_frame(&self.state, frame)?;
        validate_damage(frame, regions)?;
        validate_damage(frame, clean_regions)?;
        let has_work = !regions.is_empty() || !clean_regions.is_empty();
        let full = force_full || previous.is_none() || (has_work && policy.full_due());
        let viewport = Rect::new(0, 0, frame.width(), frame.height());
        let planned = if full {
            vec![PresentRegion::clean(viewport)]
        } else {
            plan_present_regions(regions, clean_regions, viewport)
        };
        let timing = submit_regions(&mut self.fbink, frame, previous, &planned, policy, full)?;
        if !planned.is_empty() {
            policy.completed(full);
        }
        Ok(timing)
    }
}
