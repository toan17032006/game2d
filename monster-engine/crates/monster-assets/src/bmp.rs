//! BMP decoder (uncompressed BI_RGB, 24/32 bpp) producing RGBA8.

use std::fmt;

/// BMP decode failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BmpError {
    /// Missing/invalid "BM" marker.
    NotBmp,
    /// Header cut short.
    Truncated,
    /// Only uncompressed 24/32-bit BMPs are supported.
    Unsupported { what: &'static str },
}

impl fmt::Display for BmpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BmpError::NotBmp => write!(f, "not a BMP file"),
            BmpError::Truncated => write!(f, "truncated BMP file"),
            BmpError::Unsupported { what } => write!(f, "unsupported BMP feature: {what}"),
        }
    }
}

/// Decoded image in RGBA8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBmp {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

fn read_u16le(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Decode an uncompressed BMP into RGBA8.
pub fn decode_bmp(bytes: &[u8]) -> Result<DecodedBmp, BmpError> {
    if bytes.len() < 14 || &bytes[0..2] != b"BM" {
        return Err(BmpError::NotBmp);
    }
    let pixel_offset = read_u32le(bytes, 10) as usize;

    // DIB header
    if bytes.len() < 18 {
        return Err(BmpError::Truncated);
    }
    let header_size = read_u32le(bytes, 14) as usize;
    if header_size < 40 {
        return Err(BmpError::Unsupported {
            what: "DIB header below BITMAPINFOHEADER",
        });
    }
    if bytes.len() < 14 + header_size {
        return Err(BmpError::Truncated);
    }
    let width = read_u32le(bytes, 18) as i32;
    let height_raw = read_u32le(bytes, 22) as i32;
    let bpp = read_u16le(bytes, 28);
    let compression = read_u32le(bytes, 30);
    if compression != 0 {
        return Err(BmpError::Unsupported {
            what: "compressed BMP",
        });
    }
    if bpp != 24 && bpp != 32 {
        return Err(BmpError::Unsupported {
            what: "bit depth (only 24/32)",
        });
    }
    if width <= 0 || height_raw == 0 {
        return Err(BmpError::Unsupported {
            what: "zero or negative width/height",
        });
    }
    let width = width as u32;
    let top_down = height_raw < 0;
    let height = height_raw.unsigned_abs();

    let bytes_pp = (bpp / 8) as usize;
    let row_size = ((width as usize) * bytes_pp).div_ceil(4) * 4;
    let needed = pixel_offset + row_size * height as usize;
    if bytes.len() < needed {
        return Err(BmpError::Truncated);
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for visual_row in 0..height {
        // bottom-up storage when height is positive
        let storage_row = if top_down {
            visual_row
        } else {
            height - 1 - visual_row
        };
        let row_start = pixel_offset + row_size * storage_row as usize;
        for x in 0..width as usize {
            let base = row_start + x * bytes_pp;
            let b = bytes[base];
            let g = bytes[base + 1];
            let r = bytes[base + 2];
            let a = if bpp == 32 { bytes[base + 3] } else { 255 };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }

    Ok(DecodedBmp {
        width,
        height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal 24bpp bottom-up BMP in-memory.
    fn build_bmp(
        width: u32,
        height: u32,
        top_down: bool,
        pixel_fn: impl Fn(u32, u32) -> [u8; 3],
    ) -> Vec<u8> {
        let bytes_pp = 3usize;
        let row_size = ((width as usize) * bytes_pp).div_ceil(4) * 4;
        let pixel_data = row_size * height as usize;
        let mut out = Vec::new();
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(14 + 40 + pixel_data as u32).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(14u32 + 40).to_le_bytes()); // pixel offset
                                                            // BITMAPINFOHEADER
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&width.to_le_bytes());
        let h = if top_down {
            -(height as i32)
        } else {
            height as i32
        };
        out.extend_from_slice(&h.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
        out.extend_from_slice(&(pixel_data as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        // rows bottom-up by default
        let row_pad = row_size - (width as usize) * bytes_pp;
        for storage_row in 0..height {
            let visual_row = if top_down {
                storage_row
            } else {
                height - 1 - storage_row
            };
            for x in 0..width {
                let [r, g, b] = pixel_fn(visual_row, x);
                out.push(b);
                out.push(g);
                out.push(r);
            }
            for _ in 0..row_pad {
                out.push(0); // per-row padding to the 4-byte boundary
            }
        }
        out
    }

    #[test]
    fn decodes_bottom_up_24bpp() {
        // top row red, bottom row blue
        let bmp = build_bmp(2, 2, false, |row, x| {
            if row == 0 {
                [255, 0, 0]
            } else {
                [0, 0, 255]
            }
        });
        let img = decode_bmp(&bmp).expect("decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        // visual row 0 = red (even with bottom-up storage)
        assert_eq!(&img.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&img.rgba[4..8], &[255, 0, 0, 255]);
        assert_eq!(&img.rgba[8..12], &[0, 0, 255, 255]);
    }

    #[test]
    fn decodes_top_down_negative_height() {
        let bmp = build_bmp(
            2,
            2,
            true,
            |row, _x| {
                if row == 0 {
                    [10, 200, 30]
                } else {
                    [1, 2, 3]
                }
            },
        );
        let img = decode_bmp(&bmp).expect("decode");
        assert_eq!(&img.rgba[0..4], &[10, 200, 30, 255]);
    }

    #[test]
    fn rejects_non_bmp_and_unsupported() {
        assert_eq!(decode_bmp(b"XX"), Err(BmpError::NotBmp));
        assert_eq!(decode_bmp(&[0u8; 100]), Err(BmpError::NotBmp));
        let mut bmp = build_bmp(1, 1, false, |_, _| [1, 2, 3]);
        bmp[30] = 1; // compression = BI_RLE8
        assert_eq!(
            decode_bmp(&bmp),
            Err(BmpError::Unsupported {
                what: "compressed BMP"
            })
        );
    }
}
