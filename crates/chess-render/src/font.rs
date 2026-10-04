//! Deterministic embedded Atkinson Hyperlegible text with Noto Sans fallback.
//!
//! Keeping the font data inside the renderer makes host snapshots and Kindle output
//! independent of fonts installed on either system.

use crate::{Gray8, Rect};
use fontdue::{
    layout::{
        CoordinateSystem, HorizontalAlign, Layout as TextLayout, LayoutSettings, TextStyle,
        VerticalAlign,
    },
    Font, FontSettings,
};
use std::sync::OnceLock;

const REGULAR_BYTES: &[u8] =
    include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Regular.ttf");
const BOLD_BYTES: &[u8] = include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Bold.ttf");

const PIXELS_PER_SCALE: f32 = 10.0;

#[derive(Clone, Copy)]
enum Face {
    Regular,
    Bold,
}

static REGULAR: OnceLock<Font> = OnceLock::new();
static BOLD: OnceLock<Font> = OnceLock::new();
static FALLBACK_REGULAR: OnceLock<Font> = OnceLock::new();
static FALLBACK_BOLD: OnceLock<Font> = OnceLock::new();

fn fallback_font(face: Face) -> &'static Font {
    match face {
        Face::Regular => FALLBACK_REGULAR.get_or_init(|| {
            load_font(
                include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf"),
                "Noto Sans regular",
            )
        }),
        Face::Bold => FALLBACK_BOLD.get_or_init(|| {
            load_font(
                include_bytes!("../../../assets/fonts/NotoSans-Bold.ttf"),
                "Noto Sans bold",
            )
        }),
    }
}

fn font_for_character(face: Face, character: char) -> &'static Font {
    let primary = font(face);
    if primary.lookup_glyph_index(character) != 0 || character.is_control() {
        primary
    } else {
        fallback_font(face)
    }
}

fn font(face: Face) -> &'static Font {
    match face {
        Face::Regular => REGULAR.get_or_init(|| load_font(REGULAR_BYTES, "regular")),
        Face::Bold => BOLD.get_or_init(|| load_font(BOLD_BYTES, "bold")),
    }
}

fn load_font(bytes: &'static [u8], label: &str) -> Font {
    Font::from_bytes(bytes, FontSettings::default())
        .unwrap_or_else(|error| panic!("embedded {label} font is invalid: {error}"))
}

fn text_px(scale: u32) -> f32 {
    scale.max(1) as f32 * PIXELS_PER_SCALE
}

pub(crate) fn draw_text_bold(canvas: &mut Gray8, x: u32, y: u32, text: &str, scale: u32, tone: u8) {
    draw_text_at(canvas, x, y, text, scale, tone, Face::Bold);
}

pub(crate) fn draw_text_regular_vertically_centered(
    canvas: &mut Gray8,
    x: u32,
    rect: Rect,
    text: &str,
    scale: u32,
    tone: u8,
) {
    let settings = LayoutSettings {
        x: x as f32,
        y: rect.y as f32,
        max_width: Some(rect.right().saturating_sub(x) as f32),
        max_height: Some(rect.height as f32),
        vertical_align: VerticalAlign::Middle,
        ..LayoutSettings::default()
    };
    draw_with_settings(
        canvas,
        text,
        scale,
        tone,
        Face::Regular,
        settings,
        Some(rect),
    );
}

pub(crate) fn measure_text_regular(text: &str, scale: u32) -> u32 {
    measure_text(text, scale, Face::Regular)
}

pub(crate) fn measure_text_bold(text: &str, scale: u32) -> u32 {
    measure_text(text, scale, Face::Bold)
}

pub(crate) fn text_line_height(scale: u32) -> u32 {
    scale.max(1).saturating_mul(12)
}

fn measure_text(text: &str, scale: u32, face: Face) -> u32 {
    let px = text_px(scale);
    text.chars()
        .filter(|character| !character.is_control())
        .map(|character| {
            font_for_character(face, character)
                .metrics(character, px)
                .advance_width
                .max(0.0)
        })
        .sum::<f32>()
        .ceil() as u32
}

fn draw_text_at(canvas: &mut Gray8, x: u32, y: u32, text: &str, scale: u32, tone: u8, face: Face) {
    let settings = LayoutSettings {
        x: x as f32,
        y: y as f32,
        ..LayoutSettings::default()
    };
    draw_with_settings(canvas, text, scale, tone, face, settings, None);
}

pub(crate) fn draw_text_bold_vertically_centered(
    canvas: &mut Gray8,
    x: u32,
    rect: Rect,
    text: &str,
    scale: u32,
    tone: u8,
) {
    let settings = LayoutSettings {
        x: x as f32,
        y: rect.y as f32,
        max_height: Some(rect.height as f32),
        vertical_align: VerticalAlign::Middle,
        ..LayoutSettings::default()
    };
    draw_with_settings(canvas, text, scale, tone, Face::Bold, settings, Some(rect));
}

pub(crate) fn draw_text_centered(canvas: &mut Gray8, rect: Rect, text: &str, scale: u32, tone: u8) {
    let settings = LayoutSettings {
        x: rect.x as f32,
        y: rect.y as f32,
        max_width: Some(rect.width as f32),
        max_height: Some(rect.height as f32),
        horizontal_align: HorizontalAlign::Center,
        vertical_align: VerticalAlign::Middle,
        ..LayoutSettings::default()
    };
    draw_with_settings(canvas, text, scale, tone, Face::Bold, settings, Some(rect));
}

pub(crate) fn draw_wrapped_text(canvas: &mut Gray8, rect: Rect, text: &str, scale: u32, tone: u8) {
    draw_wrapped_text_with_line_spacing(canvas, rect, text, scale, tone, 0);
}

pub(crate) fn draw_wrapped_text_with_line_spacing(
    canvas: &mut Gray8,
    rect: Rect,
    text: &str,
    scale: u32,
    tone: u8,
    extra_line_spacing: u32,
) {
    let px = text_px(scale);
    let settings = LayoutSettings {
        x: rect.x as f32,
        y: rect.y as f32,
        max_width: Some(rect.width as f32),
        max_height: Some(rect.height as f32),
        line_height: 1.0 + extra_line_spacing as f32 / px,
        ..LayoutSettings::default()
    };
    draw_with_settings(
        canvas,
        text,
        scale,
        tone,
        Face::Regular,
        settings,
        Some(rect),
    );
}

fn draw_with_settings(
    canvas: &mut Gray8,
    text: &str,
    scale: u32,
    tone: u8,
    face: Face,
    settings: LayoutSettings,
    clip: Option<Rect>,
) {
    if text.is_empty() {
        return;
    }

    let selected_font = font(face);
    let needs_fallback = text.chars().any(|character| {
        !character.is_control() && selected_font.lookup_glyph_index(character) == 0
    });
    let fonts = [
        selected_font,
        if needs_fallback {
            fallback_font(face)
        } else {
            selected_font
        },
    ];
    let mut layout = TextLayout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&settings);
    // Append runs to one layout so mixed scripts share wrapping and baselines.
    let mut start = 0;
    let mut run_font = 0;
    for (index, character) in text.char_indices() {
        let glyph_font = usize::from(
            !character.is_control() && selected_font.lookup_glyph_index(character) == 0,
        );
        if glyph_font != run_font {
            if start < index {
                layout.append(
                    &fonts,
                    &TextStyle::new(&text[start..index], text_px(scale), run_font),
                );
            }
            start = index;
            run_font = glyph_font;
        }
    }
    layout.append(
        &fonts,
        &TextStyle::new(&text[start..], text_px(scale), run_font),
    );

    for glyph in layout.glyphs() {
        if glyph.char_data.is_control() {
            continue;
        }
        let (metrics, bitmap) = fonts[glyph.font_index].rasterize_config(glyph.key);
        if metrics.width == 0 || metrics.height == 0 {
            continue;
        }
        blend_glyph(
            canvas,
            glyph.x.floor() as i32,
            glyph.y.floor() as i32,
            metrics.width,
            metrics.height,
            &bitmap,
            tone,
            clip,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn blend_glyph(
    canvas: &mut Gray8,
    origin_x: i32,
    origin_y: i32,
    width: usize,
    height: usize,
    alpha: &[u8],
    tone: u8,
    clip: Option<Rect>,
) {
    for source_y in 0..height {
        let y = origin_y.saturating_add(i32::try_from(source_y).unwrap_or(i32::MAX));
        if y < 0 {
            continue;
        }
        let Ok(y_u32) = u32::try_from(y) else {
            continue;
        };

        for source_x in 0..width {
            let x = origin_x.saturating_add(i32::try_from(source_x).unwrap_or(i32::MAX));
            if x < 0 {
                continue;
            }
            let Ok(x_u32) = u32::try_from(x) else {
                continue;
            };
            if let Some(bounds) = clip {
                if x_u32 < bounds.x
                    || x_u32 >= bounds.right()
                    || y_u32 < bounds.y
                    || y_u32 >= bounds.bottom()
                {
                    continue;
                }
            }

            let Some(old) = canvas.pixel(x_u32, y_u32) else {
                continue;
            };
            let index = source_y.saturating_mul(width).saturating_add(source_x);
            let Some(&opacity) = alpha.get(index) else {
                continue;
            };
            if opacity == 0 {
                continue;
            }

            let opacity = u32::from(opacity);
            let blended =
                (u32::from(tone) * opacity + u32::from(old) * (255 - opacity) + 127) / 255;
            canvas.set_pixel(
                x,
                y,
                u8::try_from(blended).expect("alpha blend stays within grayscale range"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{draw_text_at, draw_wrapped_text_with_line_spacing, font_for_character, Face};
    use crate::{Gray8, Rect};

    #[test]
    fn embedded_fonts_cover_russian_cyrillic() {
        for face in [Face::Regular, Face::Bold] {
            for character in
                "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя".chars()
            {
                assert_ne!(
                    font_for_character(face, character).lookup_glyph_index(character),
                    0,
                    "missing {character}"
                );
            }
        }
    }

    #[test]
    fn line_breaks_do_not_paint_missing_glyph_boxes() {
        let mut canvas = Gray8::new(180, 120, 255);
        let blank = canvas.clone();
        draw_wrapped_text_with_line_spacing(
            &mut canvas,
            Rect::new(0, 0, 180, 120),
            "\n\r\n",
            3,
            0,
            0,
        );
        assert!(
            canvas.pixels() == blank.pixels(),
            "line breaks should not paint pixels"
        );
    }

    #[test]
    fn atkinson_hyperlegible_preserves_letter_case() {
        let mut uppercase = Gray8::new(80, 60, 255);
        let mut lowercase = Gray8::new(80, 60, 255);

        draw_text_at(&mut uppercase, 4, 4, "A", 3, 0, Face::Regular);
        draw_text_at(&mut lowercase, 4, 4, "a", 3, 0, Face::Regular);

        assert_ne!(uppercase.pixels(), lowercase.pixels());
    }

    #[test]
    fn wrapped_text_supports_extra_pixel_spacing_between_lines() {
        let mut compact = Gray8::new(180, 120, 255);
        let mut spaced = Gray8::new(180, 120, 255);
        let rect = Rect::new(0, 0, 180, 120);

        draw_wrapped_text_with_line_spacing(&mut compact, rect, "Readable\nReadable", 3, 0, 0);
        draw_wrapped_text_with_line_spacing(&mut spaced, rect, "Readable\nReadable", 3, 0, 6);

        assert_ne!(compact.pixels(), spaced.pixels());
    }
}
