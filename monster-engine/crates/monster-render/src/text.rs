//! Blocky bitmap text: a compact 3x5 pixel font for HUD text (digits, A-Z
//! and a few punctuation marks), drawn straight into the framebuffer with
//! integer scaling. No TTF, no atlas — pure data.

use monster_math::{Color, Rect};

/// One glyph: 5 rows, 3 bits per row (LSB = leftmost column).
pub type Glyph = [u8; 5];

const fn g(rows: [u8; 5]) -> Glyph {
    rows
}

/// The built-in 3x5 font. Unknown characters map to a blank space.
const FONT: &[(char, Glyph)] = &[
    ('0', g([0b111, 0b101, 0b101, 0b101, 0b111])),
    ('1', g([0b010, 0b110, 0b010, 0b010, 0b111])),
    ('2', g([0b111, 0b001, 0b111, 0b100, 0b111])),
    ('3', g([0b111, 0b001, 0b111, 0b001, 0b111])),
    ('4', g([0b101, 0b101, 0b111, 0b001, 0b001])),
    ('5', g([0b111, 0b100, 0b111, 0b001, 0b111])),
    ('6', g([0b111, 0b100, 0b111, 0b101, 0b111])),
    ('7', g([0b111, 0b001, 0b001, 0b010, 0b010])),
    ('8', g([0b111, 0b101, 0b111, 0b101, 0b111])),
    ('9', g([0b111, 0b101, 0b111, 0b001, 0b111])),
    ('A', g([0b010, 0b101, 0b111, 0b101, 0b101])),
    ('B', g([0b110, 0b101, 0b110, 0b101, 0b110])),
    ('C', g([0b011, 0b100, 0b100, 0b100, 0b011])),
    ('D', g([0b110, 0b101, 0b101, 0b101, 0b110])),
    ('E', g([0b111, 0b100, 0b110, 0b100, 0b111])),
    ('F', g([0b111, 0b100, 0b110, 0b100, 0b100])),
    ('G', g([0b011, 0b100, 0b101, 0b101, 0b011])),
    ('H', g([0b101, 0b101, 0b111, 0b101, 0b101])),
    ('I', g([0b111, 0b010, 0b010, 0b010, 0b111])),
    ('J', g([0b001, 0b001, 0b001, 0b101, 0b010])),
    ('K', g([0b101, 0b101, 0b110, 0b101, 0b101])),
    ('L', g([0b100, 0b100, 0b100, 0b100, 0b111])),
    ('M', g([0b101, 0b111, 0b111, 0b101, 0b101])),
    ('N', g([0b101, 0b111, 0b111, 0b111, 0b101])),
    ('O', g([0b010, 0b101, 0b101, 0b101, 0b010])),
    ('P', g([0b110, 0b101, 0b110, 0b100, 0b100])),
    ('Q', g([0b010, 0b101, 0b101, 0b011, 0b001])),
    ('R', g([0b110, 0b101, 0b110, 0b101, 0b101])),
    ('S', g([0b011, 0b100, 0b010, 0b001, 0b110])),
    ('T', g([0b111, 0b010, 0b010, 0b010, 0b010])),
    ('U', g([0b101, 0b101, 0b101, 0b101, 0b111])),
    ('V', g([0b101, 0b101, 0b101, 0b101, 0b010])),
    ('W', g([0b101, 0b101, 0b111, 0b111, 0b101])),
    ('X', g([0b101, 0b101, 0b010, 0b101, 0b101])),
    ('Y', g([0b101, 0b101, 0b010, 0b010, 0b010])),
    ('Z', g([0b111, 0b001, 0b010, 0b100, 0b111])),
    (' ', g([0b000, 0b000, 0b000, 0b000, 0b000])),
    ('-', g([0b000, 0b000, 0b111, 0b000, 0b000])),
    (':', g([0b000, 0b010, 0b000, 0b010, 0b000])),
    ('.', g([0b000, 0b000, 0b000, 0b000, 0b010])),
    ('!', g([0b010, 0b010, 0b010, 0b000, 0b010])),
];

const GLYPH_W: u32 = 3;
const GLYPH_H: u32 = 5;
/// Advance between characters in font pixels.
pub const ADVANCE: u32 = GLYPH_W + 1;

/// Glyph rows for `c` (uppercase; unknown characters map to blank space).
pub fn glyph(c: char) -> Glyph {
    let upper = c.to_ascii_uppercase();
    FONT.iter()
        .find(|(ch, _)| *ch == upper)
        .map(|(_, glyph)| *glyph)
        .unwrap_or([0b000, 0b000, 0b000, 0b000, 0b000])
}

/// Pixel width of `text` at the given integer scale.
pub fn text_width(text: &str, scale: u32) -> u32 {
    let chars = text.chars().count() as u32;
    if chars == 0 {
        0
    } else {
        chars * ADVANCE * scale - scale // no trailing space after the last glyph
    }
}

/// Pixel height of text at the given integer scale.
pub const fn text_height(scale: u32) -> u32 {
    GLYPH_H * scale
}

/// Draw `text` into the framebuffer at screen-space `(x, y)` (top-left of
/// the first glyph), integer `scale`, using `color`. Uppercase-insensitive.
pub fn draw_text_on_framebuffer(
    fb: &mut crate::framebuffer::Framebuffer,
    x: f32,
    y: f32,
    text: &str,
    color: Color,
    scale: u32,
) {
    let scale = scale.max(1);
    let mut pen_x = x;
    for ch in text.chars() {
        let glyph = glyph(ch);
        for row in 0..GLYPH_H {
            let bits = glyph[row as usize];
            for col in 0..GLYPH_W {
                if bits & (1 << col) != 0 {
                    let px = pen_x + col as f32 * scale as f32;
                    let py = y + row as f32 * scale as f32;
                    fb.fill_rect(Rect::new(px, py, scale as f32, scale as f32), color);
                }
            }
        }
        pen_x += ADVANCE as f32 * scale as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use monster_math::approx_eq;

    #[test]
    fn glyphs_are_three_wide_five_tall() {
        for (ch, glyph) in FONT {
            for row in glyph {
                assert!(*row <= 0b111, "glyph {ch} row {row} exceeds 3 bits");
            }
        }
        assert_eq!(glyph('1'), [0b010, 0b110, 0b010, 0b010, 0b111]);
        assert_eq!(glyph('z'), glyph('Z')); // uppercase-insensitive
        assert_eq!(glyph('?'), glyph(' ')); // unknown -> blank
    }

    #[test]
    fn text_metrics() {
        assert_eq!(text_width("", 2), 0);
        assert_eq!(text_width("1", 2), ADVANCE * 2 - 2); // 12 - 2 = 10
        assert_eq!(text_width("SCORE", 1), 5 * ADVANCE - 1);
        assert_eq!(text_height(3), 15);
    }

    #[test]
    fn draw_text_writes_pixels() {
        let mut fb = Framebuffer::new(40, 12, Color::BLACK);
        draw_text_on_framebuffer(&mut fb, 2.0, 3.0, "1", Color::WHITE, 1);
        // glyph '1' at scale 1: col 1 rows 0..2 and col 0..3 row 4 (approx)
        // just assert SOME white pixels exist inside the glyph box and none outside
        let mut inside = 0;
        let mut outside = 0;
        for y in 0..12 {
            for x in 0..40 {
                let p = fb.get_pixel(x, y);
                let in_box = x >= 2 && x < 2 + 3 && y >= 3 && y < 3 + 5;
                if p[0] == 255 {
                    if in_box {
                        inside += 1;
                    } else {
                        outside += 1;
                    }
                }
            }
        }
        assert!(inside > 0, "glyph pixels drawn");
        assert_eq!(outside, 0, "no stray pixels outside the glyph box");
    }

    #[test]
    fn draw_text_respects_scale_and_advances() {
        let mut fb = Framebuffer::new(60, 20, Color::BLACK);
        draw_text_on_framebuffer(&mut fb, 1.0, 1.0, "11", Color::WHITE, 2);
        // second glyph starts at x = 1 + ADVANCE*2 = 1 + 16
        let p = fb.get_pixel(1 + ADVANCE * 2 + 2, 1 + 2); // row 1 col 1 of glyph 2nd (scale 2 block at x=11..13, y=3..5)
        assert_eq!(p[0], 255, "second glyph drawn at advance offset");
        let _ = approx_eq;
    }
}
