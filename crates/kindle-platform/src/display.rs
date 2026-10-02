//! Safe FBInk-backed display adapter for the measured Kindle Scribe target.

use chess_render::Gray8;
use fbink_sys::{FbInk, FbInkError, FbInkState};
use std::fmt;

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
            Self::UnsupportedPixelFormat => "Task 04 requires the measured 8-bit Y8 framebuffer mode",
            Self::NonTightFrame => "Task 04 FBInk presentation requires a tightly packed Gray8 frame",
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

pub fn validate_presentable_frame(
    state: &DisplayState,
    frame: &Gray8,
) -> Result<(), PresentError> {
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
        self.reinitialize()?;
        validate_presentable_frame(&self.state, frame)?;
        self.fbink
            .present_gray8(frame.width(), frame.height(), frame.pixels())?;
        Ok(())
    }
}
