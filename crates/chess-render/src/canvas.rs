//! Deterministic project-owned 8-bit grayscale canvas.

use crate::Rect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanvasError {
    InvalidStride,
    SizeOverflow,
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStride => formatter.write_str("Gray8 stride must be at least width"),
            Self::SizeOverflow => formatter.write_str("Gray8 dimensions overflow address space"),
        }
    }
}

impl std::error::Error for CanvasError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gray8 {
    width: u32,
    height: u32,
    stride: u32,
    pixels: Vec<u8>,
}

impl Gray8 {
    pub fn new(width: u32, height: u32, tone: u8) -> Self {
        Self::with_stride(width, height, width, tone)
            .expect("width-sized Gray8 stride always validates")
    }

    pub fn with_stride(
        width: u32,
        height: u32,
        stride: u32,
        tone: u8,
    ) -> Result<Self, CanvasError> {
        if stride < width {
            return Err(CanvasError::InvalidStride);
        }
        let len = usize::try_from(stride)
            .ok()
            .and_then(|stride| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| stride.checked_mul(height))
            })
            .ok_or(CanvasError::SizeOverflow)?;
        Ok(Self {
            width,
            height,
            stride,
            pixels: vec![tone; len],
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn stride(&self) -> u32 {
        self.stride
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<u8> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, tone: u8) {
        if x < 0 || y < 0 {
            return;
        }
        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
            return;
        };
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = tone;
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, tone: u8) {
        let x0 = rect.x.min(self.width);
        let y0 = rect.y.min(self.height);
        let x1 = rect.right().min(self.width);
        let y1 = rect.bottom().min(self.height);
        for y in y0..y1 {
            let row = usize::try_from(y.saturating_mul(self.stride)).expect("canvas index fits");
            for x in x0..x1 {
                self.pixels[row + usize::try_from(x).expect("canvas index fits")] = tone;
            }
        }
    }

    pub fn stroke_rect(&mut self, rect: Rect, thickness: u32, tone: u8) {
        if thickness == 0 || rect.width == 0 || rect.height == 0 {
            return;
        }
        let thickness = thickness.min(rect.width).min(rect.height);
        self.fill_rect(Rect::new(rect.x, rect.y, rect.width, thickness), tone);
        self.fill_rect(
            Rect::new(
                rect.x,
                rect.bottom().saturating_sub(thickness),
                rect.width,
                thickness,
            ),
            tone,
        );
        self.fill_rect(Rect::new(rect.x, rect.y, thickness, rect.height), tone);
        self.fill_rect(
            Rect::new(
                rect.right().saturating_sub(thickness),
                rect.y,
                thickness,
                rect.height,
            ),
            tone,
        );
    }

    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, tone: u8) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            self.set_pixel(x0, y0, tone);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let doubled = error.saturating_mul(2);
            if doubled >= dy {
                error += dy;
                x0 += sx;
            }
            if doubled <= dx {
                error += dx;
                y0 += sy;
            }
        }
    }

    pub fn blit_gray(
        &mut self,
        width: u32,
        height: u32,
        stride: u32,
        pixels: &[u8],
        dst_x: u32,
        dst_y: u32,
    ) {
        if stride < width {
            return;
        }
        let Some(required) = usize::try_from(stride).ok().and_then(|stride| {
            usize::try_from(height)
                .ok()
                .and_then(|h| stride.checked_mul(h))
        }) else {
            return;
        };
        if pixels.len() < required {
            return;
        }

        for source_y in 0..height {
            let y = dst_y.saturating_add(source_y);
            if y >= self.height {
                break;
            }
            for source_x in 0..width {
                let x = dst_x.saturating_add(source_x);
                if x >= self.width {
                    break;
                }
                let source =
                    usize::try_from(source_y.saturating_mul(stride).saturating_add(source_x))
                        .expect("source index fits");
                if let Some(destination) = self.index(x, y) {
                    self.pixels[destination] = pixels[source];
                }
            }
        }
    }

    pub fn blit_alpha(
        &mut self,
        width: u32,
        height: u32,
        stride: u32,
        alpha: &[u8],
        dst_x: u32,
        dst_y: u32,
        tone: u8,
    ) {
        if stride < width {
            return;
        }
        let Some(required) = usize::try_from(stride).ok().and_then(|stride| {
            usize::try_from(height)
                .ok()
                .and_then(|h| stride.checked_mul(h))
        }) else {
            return;
        };
        if alpha.len() < required {
            return;
        }

        for source_y in 0..height {
            let y = dst_y.saturating_add(source_y);
            if y >= self.height {
                break;
            }
            for source_x in 0..width {
                let x = dst_x.saturating_add(source_x);
                if x >= self.width {
                    break;
                }
                let source =
                    usize::try_from(source_y.saturating_mul(stride).saturating_add(source_x))
                        .expect("source index fits");
                let opacity = u32::from(alpha[source]);
                if opacity == 0 {
                    continue;
                }
                if let Some(destination) = self.index(x, y) {
                    let old = u32::from(self.pixels[destination]);
                    let blended = (u32::from(tone) * opacity + old * (255 - opacity) + 127) / 255;
                    self.pixels[destination] = u8::try_from(blended).expect("blend stays in range");
                }
            }
        }
    }

    pub(crate) fn fill_polygon(&mut self, points: &[(i32, i32)], tone: u8) {
        if points.len() < 3 {
            return;
        }
        let Some(min_y) = points.iter().map(|point| point.1).min() else {
            return;
        };
        let Some(max_y) = points.iter().map(|point| point.1).max() else {
            return;
        };

        for y in min_y.max(0)..max_y.min(i32::try_from(self.height).unwrap_or(i32::MAX)) {
            let mut intersections = Vec::new();
            for index in 0..points.len() {
                let (x0, y0) = points[index];
                let (x1, y1) = points[(index + 1) % points.len()];
                if (y0 <= y && y < y1) || (y1 <= y && y < y0) {
                    let numerator = i64::from(y - y0) * i64::from(x1 - x0);
                    let denominator = i64::from(y1 - y0);
                    let x = i64::from(x0) + numerator / denominator;
                    intersections.push(i32::try_from(x).unwrap_or(if x < 0 {
                        i32::MIN
                    } else {
                        i32::MAX
                    }));
                }
            }
            intersections.sort_unstable();
            for pair in intersections.chunks_exact(2) {
                let start = pair[0].max(0);
                let end = pair[1].min(i32::try_from(self.width).unwrap_or(i32::MAX));
                for x in start..end {
                    self.set_pixel(x, y, tone);
                }
            }
        }
    }

    pub fn checksum64(&self) -> u64 {
        const OFFSET: u64 = 0xcbf29ce484222325;
        const PRIME: u64 = 0x100000001b3;
        let mut hash = OFFSET;
        for byte in self
            .width
            .to_le_bytes()
            .into_iter()
            .chain(self.height.to_le_bytes())
            .chain(self.stride.to_le_bytes())
            .chain(self.pixels.iter().copied())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
        hash
    }

    pub fn to_pgm(&self) -> Vec<u8> {
        let mut output = format!("P5\n{} {}\n255\n", self.width, self.height).into_bytes();
        for y in 0..self.height {
            let start = usize::try_from(y.saturating_mul(self.stride)).expect("row fits");
            let end = start + usize::try_from(self.width).expect("width fits");
            output.extend_from_slice(&self.pixels[start..end]);
        }
        output
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        usize::try_from(y.saturating_mul(self.stride).saturating_add(x)).ok()
    }
}
