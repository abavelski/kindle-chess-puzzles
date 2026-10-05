//! Kindle-specific display, input, and storage adapters.

#![forbid(unsafe_code)]

mod display;
mod input;
mod power;
mod refresh;
pub use power::{complete_native_wake, PowerEvent, PowerEvents};
mod storage;

pub use display::{
    submit_regions, validate_damage, validate_presentable_frame, validate_task04_scribe_state,
    DisplayError, DisplayState, KindleDisplay, PresentError, PresentTiming, RegionPresenter,
    SCRIBE_DPI, SCRIBE_HEIGHT, SCRIBE_ROTATION, SCRIBE_WIDTH,
};
pub use input::{
    input_device_path, is_finger_touchscreen_candidate, is_scribe_pen_candidate,
    scan_input_candidates, scribe_pen_transform, select_finger_touchscreen,
    task04_scribe_transform, AxisRange, DeviceCapabilities, DeviceEvent, FingerInput,
    InputCandidate, InputError, MtDecoder, PenTapDecoder, RawInputEvent, Rotation, ScribeInput,
    TapPolicy, TapRecognizer, TouchEvent, TouchPhase, TouchTransform, TransformError,
    SCRIBE_TOUCH_X, SCRIBE_TOUCH_Y,
};
pub use storage::{
    DiscoveredCollection, KindleStorage, ProgressLoad, ProgressStore, StorageError, StoragePaths,
    DEFAULT_PROGRESS_FILE, DEFAULT_PUZZLE_DIR, PROGRESS_FILE_ENV, PUZZLE_DIR_ENV,
};

pub use refresh::{
    classify_region, clean_regions_for_board_change, pack_region, plan_present_regions,
    ContentClass, PresentRegion, RefreshMode, RefreshPolicy, RefreshStrength,
};

pub const PLATFORM_IMPLEMENTED: bool = true;
