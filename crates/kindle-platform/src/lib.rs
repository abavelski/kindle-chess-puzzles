//! Kindle-specific display and input adapters.

#![forbid(unsafe_code)]

mod display;
mod input;

pub use display::{
    validate_presentable_frame, validate_task04_scribe_state, DisplayError, DisplayState,
    KindleDisplay, PresentError, SCRIBE_DPI, SCRIBE_HEIGHT, SCRIBE_ROTATION, SCRIBE_WIDTH,
};
pub use input::{
    input_device_path, is_finger_touchscreen_candidate, scan_input_candidates,
    select_finger_touchscreen, task04_scribe_transform, AxisRange, DeviceCapabilities,
    FingerInput, InputCandidate, InputError, MtDecoder, RawInputEvent, Rotation, TapPolicy,
    TapRecognizer, TouchEvent, TouchPhase, TouchTransform, TransformError, SCRIBE_TOUCH_X,
    SCRIBE_TOUCH_Y,
};

pub const PLATFORM_IMPLEMENTED: bool = true;
