//! Tiny deterministic 5x7 bitmap font used for renderer-owned text.

use crate::{Gray8, Rect};

const GLYPH_WIDTH: u32 = 5;
const GLYPH_HEIGHT: u32 = 7;
const GLYPH_ADVANCE: u32 = 6;
const LINE_ADVANCE: u32 = 8;

pub(crate) fn measure_text(text: &str, scale: u32) -> (u32, u32) {
    let count = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
    let width = if count == 0 {
        0
    } else {
        count
            .saturating_mul(GLYPH_ADVANCE)
            .saturating_sub(1)
            .saturating_mul(scale)
    };
    (width, GLYPH_HEIGHT.saturating_mul(scale))
}

pub(crate) fn draw_text(
    canvas: &mut Gray8,
    x: u32,
    y: u32,
    text: &str,
    scale: u32,
    tone: u8,
) {
    let scale = scale.max(1);
    let mut cursor_x = x;
    for character in text.chars() {
        draw_char(canvas, cursor_x, y, character, scale, tone);
        cursor_x = cursor_x.saturating_add(GLYPH_ADVANCE.saturating_mul(scale));
    }
}

pub(crate) fn draw_text_centered(
    canvas: &mut Gray8,
    rect: Rect,
    text: &str,
    scale: u32,
    tone: u8,
) {
    let (width, height) = measure_text(text, scale);
    let x = rect.x.saturating_add(rect.width.saturating_sub(width) / 2);
    let y = rect.y.saturating_add(rect.height.saturating_sub(height) / 2);
    draw_text(canvas, x, y, text, scale, tone);
}

pub(crate) fn draw_wrapped_text(
    canvas: &mut Gray8,
    rect: Rect,
    text: &str,
    scale: u32,
    tone: u8,
) {
    let scale = scale.max(1);
    let char_width = GLYPH_ADVANCE.saturating_mul(scale);
    let max_chars = (rect.width / char_width).max(1) as usize;
    let line_height = LINE_ADVANCE.saturating_mul(scale);
    let max_lines = (rect.height / line_height).max(1) as usize;
    let lines = wrap_lines(text, max_chars, max_lines);
    for (index, line) in lines.iter().enumerate() {
        let y = rect
            .y
            .saturating_add(u32::try_from(index).unwrap_or(u32::MAX).saturating_mul(line_height));
        draw_text(canvas, rect.x, y, line, scale, tone);
    }
}

fn wrap_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let extra = usize::from(!current.is_empty()) + word.chars().count();
            if !current.is_empty() && current.chars().count().saturating_add(extra) > max_chars {
                lines.push(current);
                if lines.len() >= max_lines {
                    return lines;
                }
                current = String::new();
            }

            if word.chars().count() > max_chars {
                if !current.is_empty() {
                    lines.push(current);
                    if lines.len() >= max_lines {
                        return lines;
                    }
                    current = String::new();
                }
                let chars: Vec<char> = word.chars().collect();
                for chunk in chars.chunks(max_chars) {
                    lines.push(chunk.iter().collect());
                    if lines.len() >= max_lines {
                        return lines;
                    }
                }
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        } else if paragraph.is_empty() {
            lines.push(String::new());
        }
        if lines.len() >= max_lines {
            break;
        }
    }
    lines.truncate(max_lines);
    lines
}

fn draw_char(canvas: &mut Gray8, x: u32, y: u32, character: char, scale: u32, tone: u8) {
    let rows = glyph(character);
    for (row, bits) in rows.into_iter().enumerate() {
        for column in 0..GLYPH_WIDTH {
            let mask = 1_u8 << (GLYPH_WIDTH - 1 - column);
            if bits & mask == 0 {
                continue;
            }
            let px = x.saturating_add(column.saturating_mul(scale));
            let py = y.saturating_add(
                u32::try_from(row)
                    .unwrap_or(u32::MAX)
                    .saturating_mul(scale),
            );
            canvas.fill_rect(Rect::new(px, py, scale, scale), tone);
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110],
        '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '.' => [0, 0, 0, 0, 0, 0b00110, 0b00110],
        ':' => [0, 0b00110, 0b00110, 0, 0b00110, 0b00110, 0],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '<' => [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
        '>' => [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0, 0b00100],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '=' => [0, 0b11111, 0, 0b11111, 0, 0, 0],
        ' ' => [0; 7],
        _ => [0b11111, 0b10001, 0b00010, 0b00100, 0b00100, 0, 0b00100],
    }
}
