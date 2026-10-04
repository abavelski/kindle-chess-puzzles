//! Narrow Rust boundary around the pinned FBInk C library.
//!
//! The application never depends on FBInk structs or raw framebuffer ioctls.
//! All unsafe FFI calls are contained in this crate.

use std::fmt;
mod input;
pub use input::{wait_input, wait_input_power, ExclusiveInput};

pub const PINNED_FBINK_REVISION: &str = env!("KCP_FBINK_REVISION");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    EmptyFrame,
    DimensionTooLarge,
    BufferLength,
    EmptyRect,
    RectOutOfBounds,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyFrame => "frame dimensions must be non-zero",
            Self::DimensionTooLarge => "dimensions or offsets exceed FBInk's signed integer API",
            Self::BufferLength => "Gray8 buffer length does not match width times height",
            Self::EmptyRect => "rectangle dimensions must be non-zero",
            Self::RectOutOfBounds => "rectangle lies outside the framebuffer",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_frame(width: u32, height: u32, len: usize) -> Result<(), ValidationError> {
    if width == 0 || height == 0 {
        return Err(ValidationError::EmptyFrame);
    }
    if width > i32::MAX as u32 || height > i32::MAX as u32 {
        return Err(ValidationError::DimensionTooLarge);
    }
    let expected = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(ValidationError::BufferLength)?;
    if expected != len {
        return Err(ValidationError::BufferLength);
    }
    Ok(())
}

/// Raw-data destination offsets use signed short integers in the pinned API.
pub fn validate_offset(left: u32, top: u32) -> Result<(), ValidationError> {
    if left > i16::MAX as u32 || top > i16::MAX as u32 {
        return Err(ValidationError::DimensionTooLarge);
    }
    Ok(())
}

pub fn validate_rect(
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    screen_width: u32,
    screen_height: u32,
) -> Result<(), ValidationError> {
    if width == 0 || height == 0 {
        return Err(ValidationError::EmptyRect);
    }
    let right = left
        .checked_add(width)
        .ok_or(ValidationError::RectOutOfBounds)?;
    let bottom = top
        .checked_add(height)
        .ok_or(ValidationError::RectOutOfBounds)?;
    if right > screen_width || bottom > screen_height {
        return Err(ValidationError::RectOutOfBounds);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FbInkState {
    pub width: u32,
    pub height: u32,
    pub scanline_stride: u32,
    pub bpp: u32,
    pub rotation: u8,
    pub is_y8: bool,
}

#[derive(Debug)]
pub enum FbInkError {
    UnsupportedPlatform,
    Validation(ValidationError),
    Call { operation: &'static str, code: i32 },
}

impl fmt::Display for FbInkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter
                .write_str("FBInk runtime is only linked for armv7-unknown-linux-gnueabihf"),
            Self::Validation(error) => write!(formatter, "invalid FBInk request: {error}"),
            Self::Call { operation, code } => {
                write!(formatter, "{operation} failed with FBInk error {code}")
            }
        }
    }
}

impl std::error::Error for FbInkError {}

impl From<ValidationError> for FbInkError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

/// Values are project-owned; the C bridge maps to pinned FBInk enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RefreshMode {
    Full = 0,
    AutoPartial = 1,
    GrayPartial = 2,
    FastMono = 3,
    Clean = 4,
}

#[derive(Debug)]
pub struct FbInk {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    fd: i32,
}

impl FbInk {
    pub fn open() -> Result<Self, FbInkError> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            let fd = unsafe { ffi::kcp_fbink_open() };
            if fd < 0 {
                return Err(FbInkError::Call {
                    operation: "fbink_open/init",
                    code: fd,
                });
            }
            Ok(Self { fd })
        }

        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            Err(FbInkError::UnsupportedPlatform)
        }
    }

    pub fn reinitialize(&mut self) -> Result<(), FbInkError> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            let code = unsafe { ffi::kcp_fbink_reinit(self.fd) };
            call_result("fbink_reinit", code)
        }

        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            Err(FbInkError::UnsupportedPlatform)
        }
    }

    pub fn state(&self) -> Result<FbInkState, FbInkError> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            let mut raw = ffi::RawState::default();
            let code = unsafe { ffi::kcp_fbink_get_state(&mut raw) };
            call_result("fbink_get_state", code)?;
            Ok(FbInkState {
                width: raw.width,
                height: raw.height,
                scanline_stride: raw.scanline_stride,
                bpp: raw.bpp,
                rotation: raw.rotation,
                is_y8: raw.is_y8 != 0,
            })
        }

        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            Err(FbInkError::UnsupportedPlatform)
        }
    }

    pub fn present_gray8(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), FbInkError> {
        self.present_region(0, 0, width, height, pixels, RefreshMode::Full)?;
        self.wait_for_complete()
    }

    pub fn present_region(
        &mut self,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        pixels: &[u8],
        mode: RefreshMode,
    ) -> Result<(), FbInkError> {
        validate_frame(width, height, pixels.len())?;
        validate_offset(left, top)?;
        let state = self.state()?;
        validate_rect(left, top, width, height, state.width, state.height)?;
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            let code = unsafe {
                ffi::kcp_fbink_present_region(
                    self.fd,
                    pixels.as_ptr(),
                    width,
                    height,
                    pixels.len(),
                    left,
                    top,
                    mode as u8,
                )
            };
            call_result("fbink_print_raw_data/region", code)
        }
        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            let _ = mode;
            Err(FbInkError::UnsupportedPlatform)
        }
    }

    pub fn wait_for_complete(&mut self) -> Result<(), FbInkError> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            call_result("fbink_wait_for_complete", unsafe {
                ffi::kcp_fbink_wait(self.fd)
            })
        }
        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            Err(FbInkError::UnsupportedPlatform)
        }
    }

    pub fn version(&self) -> Result<String, FbInkError> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            use std::ffi::CStr;
            let pointer = unsafe { ffi::kcp_fbink_version() };
            if pointer.is_null() {
                return Err(FbInkError::Call {
                    operation: "fbink_version",
                    code: -1,
                });
            }
            let version = unsafe { CStr::from_ptr(pointer) };
            Ok(version.to_string_lossy().into_owned())
        }

        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            Ok(PINNED_FBINK_REVISION.to_owned())
        }
    }
}

impl Drop for FbInk {
    fn drop(&mut self) {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            if self.fd >= 0 {
                let _ = unsafe { ffi::kcp_fbink_close(self.fd) };
                self.fd = -1;
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
fn call_result(operation: &'static str, code: i32) -> Result<(), FbInkError> {
    if code < 0 {
        Err(FbInkError::Call { operation, code })
    } else {
        Ok(())
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
mod ffi {
    use std::os::raw::{c_char, c_int};

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct RawState {
        pub width: u32,
        pub height: u32,
        pub scanline_stride: u32,
        pub bpp: u32,
        pub rotation: u8,
        pub is_y8: u8,
    }

    extern "C" {
        pub fn kcp_fbink_open() -> c_int;
        pub fn kcp_fbink_reinit(fbfd: c_int) -> c_int;
        pub fn kcp_fbink_get_state(out: *mut RawState) -> c_int;
        pub fn kcp_fbink_present_region(
            fbfd: c_int,
            data: *const u8,
            width: u32,
            height: u32,
            len: usize,
            left: u32,
            top: u32,
            mode: u8,
        ) -> c_int;
        pub fn kcp_fbink_wait(fbfd: c_int) -> c_int;
        pub fn kcp_fbink_close(fbfd: c_int) -> c_int;
        pub fn kcp_fbink_version() -> *const c_char;
    }
}
