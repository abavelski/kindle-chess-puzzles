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

pub(crate) fn draw_text(canvas: &mut Gray8, x: u32, y: u32, text: &str, scale: u32, tone: u8) {
    let scale = scale.max(1);
    let mut cursor_x = x;
    for character in text.chars() {
        draw_char(canvas, cursor_x, y, character, scale, tone);
        cursor_x = cursor_x.saturating_add(GLYPH_ADVANCE.saturating_mul(scale));
    }
}

pub(crate) fn draw_text_centered(canvas: &mut Gray8, rect: Rect, text: &str, scale: u32, tone: u8) {
    let (width, height) = measure_text(text, scale);
    let x = rect.x.saturating_add(rect.width.saturating_sub(width) / 2);
    let y = rect
        .y
        .saturating_add(rect.height.saturating_sub(height) / 2);
    draw_text(canvas, x, y, text, scale, tone);
}

pub(crate) fn draw_wrapped_text(canvas: &mut Gray8, rect: Rect, text: &str, scale: u32, tone: u8) {
    let scale = scale.max(1);
    let char_width = GLYPH_ADVANCE.saturating_mul(scale);
    let max_chars = (rect.width / char_width).max(1) as usize;
    let line_height = LINE_ADVANCE.saturating_mul(scale);
    let max_lines = (rect.height / line_height).max(1) as usize;
    let lines = wrap_lines(text, max_chars, max_lines);
    for (index, line) in lines.iter().enumerate() {
        let y = rect.y.saturating_add(
            u32::try_from(index)
                .unwrap_or(u32::MAX)
                .saturating_mul(line_height),
        );
        draw_text(canvas, rect.x, y, line, scale, tone);
    }
}

fn wrap_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let extra = if current.is_empty() { 0 } else { 1 } + word.chars().count();
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
    let pixels = glyph(character);
    for row in 0..GLYPH_HEIGHT {
        for column in 0..GLYPH_WIDTH {
            let index = usize::try_from(row * GLYPH_WIDTH + column).expect("glyph index fits");
            if pixels[index] != b'1' {
                continue;
            }
            let px = x.saturating_add(column.saturating_mul(scale));
            let py = y.saturating_add(row.saturating_mul(scale));
            canvas.fill_rect(Rect::new(px, py, scale, scale), tone);
        }
    }
}

fn glyph(character: char) -> &'static [u8; 35] {
    match character.to_ascii_uppercase() {
        'A' => b"01110100011000111111100011000110001",
        'B' => b"11110100011000111110100011000111110",
        'C' => b"01111100001000010000100001000001111",
        'D' => b"11110100011000110001100011000111110",
        'E' => b"11111100001000011110100001000011111",
        'F' => b"11111100001000011110100001000010000",
        'G' => b"01111100001000010111100011000101111",
        'H' => b"10001100011000111111100011000110001",
        'I' => b"11111001000010000100001000010011111",
        'J' => b"00111000100001000010100101001001100",
        'K' => b"10001100101010011000101001001010001",
        'L' => b"10000100001000010000100001000011111",
        'M' => b"10001110111010110101100011000110001",
        'N' => b"10001110011010110011100011000110001",
        'O' => b"01110100011000110001100011000101110",
        'P' => b"11110100011000111110100001000010000",
        'Q' => b"01110100011000110001101011001001101",
        'R' => b"11110100011000111110101001001010001",
        'S' => b"01111100001000001110000010000111110",
        'T' => b"11111001000010000100001000010000100",
        'U' => b"10001100011000110001100011000101110",
        'V' => b"10001100011000110001100010101000100",
        'W' => b"10001100011000110101101011010101010",
        'X' => b"10001100010101000100010101000110001",
        'Y' => b"10001100010101000100001000010000100",
        'Z' => b"11111000010001000100010001000011111",
        '0' => b"01110100011001110101110011000101110",
        '1' => b"00100011000010000100001000010001110",
        '2' => b"01110100010000100010001000100011111",
        '3' => b"11110000010000101110000010000111110",
        '4' => b"00010001100101010010111110001000010",
        '5' => b"11111100001000011110000010000111110",
        '6' => b"01110100001000011110100011000101110",
        '7' => b"11111000010001000100010000100001000",
        '8' => b"01110100011000101110100011000101110",
        '9' => b"01110100011000101111000010000101110",
        '-' => b"00000000000000011111000000000000000",
        '_' => b"00000000000000000000000000000011111",
        '/' => b"00001000100001000100010000100010000",
        '.' => b"00000000000000000000000000011000110",
        ':' => b"00000001100011000000001100011000000",
        '(' => b"00010001000100001000010000010000010",
        ')' => b"01000001000001000010000100010001000",
        '[' => b"01110010000100001000010000100001110",
        ']' => b"01110000100001000010000100001001110",
        '<' => b"00010001000100010000010000010000010",
        '>' => b"01000001000001000001000100010001000",
        '!' => b"00100001000010000100001000000000100",
        '?' => b"01110100010000100010001000000000100",
        '+' => b"00000001000010011111001000010000000",
        '=' => b"00000111110000011111000000000000000",
        ' ' => b"00000000000000000000000000000000000",
        _ => b"11111100010001000100001000000000100",
    }
}


#[cfg(test)]
mod tests {
    use super::draw_wrapped_text_with_line_spacing;
    use crate::{Gray8, Rect};

    #[test]
    fn wrapped_text_supports_extra_pixel_spacing_between_lines() {
        let mut canvas = Gray8::new(24, 24, 255);
        draw_wrapped_text_with_line_spacing(
            &mut canvas,
            Rect::new(0, 0, 24, 24),
            "A\nA",
            1,
            0,
            3,
        );

        for y in 7..11 {
            assert!(
                (0..24).all(|x| canvas.pixel(x, y) == Some(255)),
                "expected blank separator row at y={y}"
            );
        }
    }
}
