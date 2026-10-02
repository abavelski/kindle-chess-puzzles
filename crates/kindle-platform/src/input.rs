//! Kindle finger-touch discovery, decoding, normalization, and tap recognition.

use chess_render::DisplayMetrics;
use std::{
    fmt,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

pub const SCRIBE_TOUCH_X: AxisRange = AxisRange { min: 0, max: 1859 };
pub const SCRIBE_TOUCH_Y: AxisRange = AxisRange { min: 0, max: 2479 };

const EV_SYN: u16 = 0;
const EV_ABS: u16 = 3;
const SYN_REPORT: u16 = 0;
const ABS_MT_POSITION_X: u16 = 0x35;
const ABS_MT_POSITION_Y: u16 = 0x36;
const ABS_MT_TRACKING_ID: u16 = 0x39;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisRange {
    min: i32,
    max: i32,
}

impl AxisRange {
    pub const fn new(min: i32, max: i32) -> Option<Self> {
        if max > min {
            Some(Self { min, max })
        } else {
            None
        }
    }

    fn normalize(self, value: i32, extent: u32) -> u32 {
        if extent <= 1 {
            return 0;
        }
        let clamped = value.clamp(self.min, self.max);
        let numerator = i64::from(clamped - self.min) * i64::from(extent - 1);
        let denominator = i64::from(self.max - self.min);
        u32::try_from((numerator + denominator / 2) / denominator).unwrap_or(extent - 1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TouchTransform {
    raw_x: AxisRange,
    raw_y: AxisRange,
    metrics: DisplayMetrics,
    rotation: Rotation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformError {
    GeometryMismatch,
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("display geometry does not match the requested touch rotation")
    }
}

impl std::error::Error for TransformError {}

impl TouchTransform {
    pub fn new(
        raw_x: AxisRange,
        raw_y: AxisRange,
        metrics: DisplayMetrics,
        rotation: Rotation,
    ) -> Result<Self, TransformError> {
        let raw_width = u32::try_from(raw_x.max - raw_x.min + 1).unwrap_or(0);
        let raw_height = u32::try_from(raw_y.max - raw_y.min + 1).unwrap_or(0);
        let expected = match rotation {
            Rotation::Deg0 | Rotation::Deg180 => (raw_width, raw_height),
            Rotation::Deg90 | Rotation::Deg270 => (raw_height, raw_width),
        };
        if (metrics.width, metrics.height) != expected {
            return Err(TransformError::GeometryMismatch);
        }
        Ok(Self {
            raw_x,
            raw_y,
            metrics,
            rotation,
        })
    }

    pub fn map(self, raw_x: i32, raw_y: i32) -> (u32, u32) {
        let source_width = match self.rotation {
            Rotation::Deg0 | Rotation::Deg180 => self.metrics.width,
            Rotation::Deg90 | Rotation::Deg270 => self.metrics.height,
        };
        let source_height = match self.rotation {
            Rotation::Deg0 | Rotation::Deg180 => self.metrics.height,
            Rotation::Deg90 | Rotation::Deg270 => self.metrics.width,
        };
        let x = self.raw_x.normalize(raw_x, source_width);
        let y = self.raw_y.normalize(raw_y, source_height);
        match self.rotation {
            Rotation::Deg0 => (x, y),
            Rotation::Deg90 => (source_height - 1 - y, x),
            Rotation::Deg180 => (source_width - 1 - x, source_height - 1 - y),
            Rotation::Deg270 => (y, source_width - 1 - x),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceCapabilities {
    pub event_types_low64: u64,
    pub abs_low64: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputCandidate {
    pub event_name: String,
    pub name: String,
    pub capabilities: DeviceCapabilities,
}

impl InputCandidate {
    pub fn diagnostic(&self) -> String {
        format!(
            "{} name={:?} ev_low64=0x{:x} abs_low64=0x{:x} finger={}",
            self.event_name,
            self.name,
            self.capabilities.event_types_low64,
            self.capabilities.abs_low64,
            is_finger_touchscreen_candidate(self)
        )
    }
}

pub fn is_finger_touchscreen_candidate(candidate: &InputCandidate) -> bool {
    let lower_name = candidate.name.to_ascii_lowercase();
    if lower_name.contains("wacom") || lower_name.contains("stylus") || lower_name.contains("pen") {
        return false;
    }

    has_bit(
        candidate.capabilities.event_types_low64,
        u8::try_from(EV_ABS).unwrap(),
    ) && has_bit(
        candidate.capabilities.abs_low64,
        u8::try_from(ABS_MT_POSITION_X).unwrap(),
    ) && has_bit(
        candidate.capabilities.abs_low64,
        u8::try_from(ABS_MT_POSITION_Y).unwrap(),
    ) && has_bit(
        candidate.capabilities.abs_low64,
        u8::try_from(ABS_MT_TRACKING_ID).unwrap(),
    )
}

fn has_bit(bits: u64, code: u8) -> bool {
    code < 64 && bits & (1_u64 << code) != 0
}

fn parse_low64_bitmap(value: &str) -> Option<u64> {
    parse_bitmap_words(value, usize::BITS)
}

fn parse_bitmap_words(value: &str, word_bits: u32) -> Option<u64> {
    // sysfs emits highest-word first, using the kernel's unsigned-long width.
    // The Scribe kernel and userspace are both 32-bit; MT axes span two words.
    let words_to_read = match word_bits {
        32 => 2,
        64 => 1,
        _ => return None,
    };
    let mut words = value.split_whitespace().rev();
    let low = u64::from_str_radix(words.next()?, 16).ok()?;
    if words_to_read == 1 {
        return Some(low);
    }
    let low = u64::from(u32::try_from(low).ok()?);
    let high = match words.next() {
        Some(word) => u64::from(u32::from_str_radix(word, 16).ok()?),
        None => 0,
    };
    Some(low | (high << 32))
}

pub fn scan_input_candidates(root: &Path) -> io::Result<Vec<InputCandidate>> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let event_name = entry.file_name().to_string_lossy().into_owned();
        if !event_name.starts_with("event") {
            continue;
        }
        let device = entry.path().join("device");
        let name = fs::read_to_string(device.join("name"))?.trim().to_owned();
        let capabilities = device.join("capabilities");
        let event_types_low64 =
            parse_low64_bitmap(&fs::read_to_string(capabilities.join("ev"))?).unwrap_or(0);
        let abs_low64 =
            parse_low64_bitmap(&fs::read_to_string(capabilities.join("abs"))?).unwrap_or(0);
        candidates.push(InputCandidate {
            event_name,
            name,
            capabilities: DeviceCapabilities {
                event_types_low64,
                abs_low64,
            },
        });
    }
    candidates.sort_by(|left, right| left.event_name.cmp(&right.event_name));
    Ok(candidates)
}

pub fn select_finger_touchscreen(
    candidates: &[InputCandidate],
) -> Result<&InputCandidate, InputError> {
    let matches = candidates
        .iter()
        .filter(|candidate| is_finger_touchscreen_candidate(candidate))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [candidate] => Ok(*candidate),
        [] => Err(InputError::NoFingerTouchscreen),
        _ => Err(InputError::AmbiguousFingerTouchscreen(
            matches
                .iter()
                .map(|candidate| candidate.event_name.clone())
                .collect(),
        )),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawInputEvent {
    pub sec: u32,
    pub usec: u32,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

impl RawInputEvent {
    pub const fn new(sec: u32, usec: u32, event_type: u16, code: u16, value: i32) -> Self {
        Self {
            sec,
            usec,
            event_type,
            code,
            value,
        }
    }

    fn time_ms(self) -> u64 {
        u64::from(self.sec)
            .saturating_mul(1000)
            .saturating_add(u64::from(self.usec) / 1000)
    }

    fn from_32bit_bytes(bytes: [u8; 16]) -> Self {
        Self {
            sec: u32::from_le_bytes(bytes[0..4].try_into().expect("slice length")),
            usec: u32::from_le_bytes(bytes[4..8].try_into().expect("slice length")),
            event_type: u16::from_le_bytes(bytes[8..10].try_into().expect("slice length")),
            code: u16::from_le_bytes(bytes[10..12].try_into().expect("slice length")),
            value: i32::from_le_bytes(bytes[12..16].try_into().expect("slice length")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchPhase {
    Down,
    Move,
    Up,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TouchEvent {
    pub phase: TouchPhase,
    pub x: i32,
    pub y: i32,
    pub time_ms: u64,
}

impl TouchEvent {
    pub const fn new(phase: TouchPhase, x: i32, y: i32, time_ms: u64) -> Self {
        Self {
            phase,
            x,
            y,
            time_ms,
        }
    }
}

#[derive(Default)]
pub struct MtDecoder {
    active: bool,
    pending_down: bool,
    pending_up: bool,
    x: Option<i32>,
    y: Option<i32>,
    last_emitted: Option<(i32, i32)>,
    frame_time_ms: u64,
}

impl MtDecoder {
    pub fn push(&mut self, event: RawInputEvent) -> Option<TouchEvent> {
        self.frame_time_ms = event.time_ms();
        if event.event_type == EV_ABS {
            match event.code {
                ABS_MT_TRACKING_ID if event.value >= 0 => {
                    self.active = false;
                    self.pending_down = true;
                    self.pending_up = false;
                    self.x = None;
                    self.y = None;
                    self.last_emitted = None;
                }
                ABS_MT_TRACKING_ID => {
                    if self.active || self.pending_down {
                        self.pending_up = true;
                    }
                }
                ABS_MT_POSITION_X => self.x = Some(event.value),
                ABS_MT_POSITION_Y => self.y = Some(event.value),
                _ => {}
            }
            return None;
        }

        if event.event_type != EV_SYN || event.code != SYN_REPORT {
            return None;
        }

        let position = self.x.zip(self.y);
        if self.pending_down {
            if let Some((x, y)) = position {
                self.pending_down = false;
                self.active = true;
                self.last_emitted = Some((x, y));
                return Some(TouchEvent::new(TouchPhase::Down, x, y, self.frame_time_ms));
            }
        }

        if self.pending_up {
            self.pending_up = false;
            let was_active = self.active;
            self.active = false;
            if was_active {
                let (x, y) = position.or(self.last_emitted)?;
                self.last_emitted = None;
                return Some(TouchEvent::new(TouchPhase::Up, x, y, self.frame_time_ms));
            }
        }

        if self.active {
            if let Some((x, y)) = position {
                if self.last_emitted != Some((x, y)) {
                    self.last_emitted = Some((x, y));
                    return Some(TouchEvent::new(TouchPhase::Move, x, y, self.frame_time_ms));
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TapPolicy {
    pub max_movement_px: u32,
    pub max_duration_ms: u64,
}

impl TapPolicy {
    pub const fn scribe_default() -> Self {
        Self {
            max_movement_px: 48,
            max_duration_ms: 750,
        }
    }
}

pub struct TapRecognizer {
    policy: TapPolicy,
    start: Option<(i32, i32, u64)>,
    cancelled: bool,
}

impl TapRecognizer {
    pub const fn new(policy: TapPolicy) -> Self {
        Self {
            policy,
            start: None,
            cancelled: false,
        }
    }

    pub fn push(&mut self, event: TouchEvent) -> Option<(i32, i32)> {
        match event.phase {
            TouchPhase::Down => {
                self.start = Some((event.x, event.y, event.time_ms));
                self.cancelled = false;
                None
            }
            TouchPhase::Move => {
                self.cancel_if_moved(event.x, event.y);
                None
            }
            TouchPhase::Up => {
                self.cancel_if_moved(event.x, event.y);
                let start = self.start.take();
                let cancelled = self.cancelled;
                self.cancelled = false;
                let (_, _, started_at) = start?;
                if cancelled
                    || event.time_ms.saturating_sub(started_at) > self.policy.max_duration_ms
                {
                    None
                } else {
                    Some((event.x, event.y))
                }
            }
        }
    }

    fn cancel_if_moved(&mut self, x: i32, y: i32) {
        let Some((start_x, start_y, _)) = self.start else {
            return;
        };
        let dx = i64::from(x) - i64::from(start_x);
        let dy = i64::from(y) - i64::from(start_y);
        let limit = i64::from(self.policy.max_movement_px);
        if dx * dx + dy * dy > limit * limit {
            self.cancelled = true;
        }
    }
}

#[derive(Debug)]
pub enum InputError {
    Io(io::Error),
    NoFingerTouchscreen,
    AmbiguousFingerTouchscreen(Vec<String>),
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "input I/O: {error}"),
            Self::NoFingerTouchscreen => {
                formatter.write_str("no unique finger touchscreen matched name/capabilities")
            }
            Self::AmbiguousFingerTouchscreen(events) => write!(
                formatter,
                "multiple finger touchscreen candidates matched: {}",
                events.join(", ")
            ),
        }
    }
}

impl std::error::Error for InputError {}

impl From<io::Error> for InputError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct FingerInput {
    file: File,
    decoder: MtDecoder,
    transform: TouchTransform,
    taps: TapRecognizer,
    selected: InputCandidate,
    diagnostics: Vec<String>,
}

impl FingerInput {
    pub fn discover(transform: TouchTransform, policy: TapPolicy) -> Result<Self, InputError> {
        Self::discover_in(
            Path::new("/sys/class/input"),
            Path::new("/dev/input"),
            transform,
            policy,
        )
    }

    pub fn discover_in(
        sys_root: &Path,
        dev_root: &Path,
        transform: TouchTransform,
        policy: TapPolicy,
    ) -> Result<Self, InputError> {
        let candidates = scan_input_candidates(sys_root)?;
        let selected = select_finger_touchscreen(&candidates)?.clone();
        let diagnostics = candidates
            .iter()
            .map(InputCandidate::diagnostic)
            .collect::<Vec<_>>();
        let file = File::open(dev_root.join(&selected.event_name))?;
        Ok(Self {
            file,
            decoder: MtDecoder::default(),
            transform,
            taps: TapRecognizer::new(policy),
            selected,
            diagnostics,
        })
    }

    pub fn selected(&self) -> &InputCandidate {
        &self.selected
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn next_tap(&mut self) -> Result<(u32, u32), InputError> {
        loop {
            let mut bytes = [0_u8; 16];
            self.file.read_exact(&mut bytes)?;
            let raw = RawInputEvent::from_32bit_bytes(bytes);
            let Some(touch) = self.decoder.push(raw) else {
                continue;
            };
            let Some((raw_x, raw_y)) = self.taps.push(touch) else {
                continue;
            };
            return Ok(self.transform.map(raw_x, raw_y));
        }
    }
}

pub fn task04_scribe_transform(metrics: DisplayMetrics) -> Result<TouchTransform, TransformError> {
    TouchTransform::new(SCRIBE_TOUCH_X, SCRIBE_TOUCH_Y, metrics, Rotation::Deg0)
}

pub fn input_device_path(dev_root: &Path, candidate: &InputCandidate) -> PathBuf {
    dev_root.join(&candidate.event_name)
}

#[cfg(test)]
mod bitmap_tests {
    use super::*;

    #[test]
    fn measured_scribe_32bit_capability_words_identify_finger_touchscreen() {
        let candidate = InputCandidate {
            event_name: "event4".into(),
            name: "pt_mt".into(),
            capabilities: DeviceCapabilities {
                event_types_low64: parse_bitmap_words("f\n", 32).unwrap(),
                abs_low64: parse_bitmap_words("ee18000 0\n", 32).unwrap(),
            },
        };
        assert_eq!(candidate.capabilities.abs_low64, 0x0ee1_8000_0000_0000);
        assert!(is_finger_touchscreen_candidate(&candidate));
    }

    #[test]
    fn bitmap_parser_keeps_low64_for_32bit_and_64bit_kernel_words() {
        assert_eq!(
            parse_bitmap_words("1 ee18000 0", 32),
            Some(0x0ee1_8000_0000_0000)
        );
        assert_eq!(
            parse_bitmap_words("1 ee1800000000000", 64),
            Some(0x0ee1_8000_0000_0000)
        );
        assert_eq!(parse_bitmap_words("0", 32), Some(0));
        assert_eq!(parse_bitmap_words("", 32), None);
        assert_eq!(parse_bitmap_words("invalid 0", 32), None);
    }
}
