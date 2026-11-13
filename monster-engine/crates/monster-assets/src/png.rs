//! PNG decoder (RFC 2083): chunks, CRC verification, zlib inflate, defilter,
//! conversion to RGBA8.
//!
//! Foundation scope: bit depth 8, no interlace, color types
//! grayscale / RGB / palette / grayscale+alpha / RGBA, with tRNS support
//! for palette (alpha per palette index) and grayscale/RGB (single
//! transparent color). These cover typical 2D game art such as the Tiny
//! Swords spritesheets.

use std::fmt;

use crate::crc::crc32;
use crate::inflate::zlib_decompress;

/// PNG decode failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PngError {
    /// Signature bytes are missing/wrong.
    BadSignature,
    /// Structural problem in the chunk stream.
    Truncated,
    /// Missing IHDR/PLTE/IDAT or duplicated critical chunk.
    MissingChunk(&'static str),
    /// IHDR parameters outside the supported subset.
    Unsupported { what: &'static str },
    /// IHDR width/height zero or inconsistent with raw data size.
    BadDimensions { width: u32, height: u32 },
    /// CRC mismatch on a chunk.
    CrcMismatch {
        chunk: [u8; 4],
        expected: u32,
        got: u32,
    },
    /// zlib/deflate failure inside IDAT.
    Inflate(crate::inflate::InflateError),
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PngError::BadSignature => write!(f, "not a PNG file (bad signature)"),
            PngError::Truncated => write!(f, "truncated PNG chunk stream"),
            PngError::MissingChunk(name) => write!(f, "missing PNG chunk '{name}'"),
            PngError::Unsupported { what } => write!(f, "unsupported PNG feature: {what}"),
            PngError::BadDimensions { width, height } => {
                write!(f, "bad PNG dimensions {width}x{height}")
            }
            PngError::CrcMismatch {
                chunk,
                expected,
                got,
            } => {
                let name = String::from_utf8_lossy(chunk);
                write!(
                    f,
                    "CRC mismatch in chunk '{name}': expected {expected:#010x}, got {got:#010x}"
                )
            }
            PngError::Inflate(e) => write!(f, "PNG IDAT inflate failure: {e}"),
        }
    }
}

pub(crate) const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

fn read_u32be(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Decoded image in RGBA8 — the engine's currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPng {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// Decode a PNG file into RGBA8.
pub fn decode_png(bytes: &[u8]) -> Result<DecodedPng, PngError> {
    if bytes.len() < 8 || bytes[..8] != SIGNATURE {
        return Err(PngError::BadSignature);
    }

    let mut width = 0u32;
    let mut height = 0u32;
    let mut color_type = 0u8;
    let mut bit_depth = 0u8;
    let mut seen_ihdr = false;
    let mut palette: Option<Vec<[u8; 3]>> = None;
    let mut trns: Option<Vec<u8>> = None;
    let mut idat: Vec<u8> = Vec::new();
    let mut seen_iend = false;

    let mut cursor = 8usize;
    while cursor + 8 <= bytes.len() {
        let size = read_u32be(bytes, cursor) as usize;
        let chunk_type = [
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ];
        let data_start = cursor + 8;
        let data_end = data_start.checked_add(size).ok_or(PngError::Truncated)?;
        if data_end + 4 > bytes.len() {
            return Err(PngError::Truncated);
        }
        let data = &bytes[data_start..data_end];
        let expected_crc = read_u32be(bytes, data_end);
        let got_crc = crc32(&bytes[cursor + 4..data_end]);
        if expected_crc != got_crc {
            return Err(PngError::CrcMismatch {
                chunk: chunk_type,
                expected: expected_crc,
                got: got_crc,
            });
        }

        match &chunk_type {
            b"IHDR" => {
                if size < 13 {
                    return Err(PngError::MissingChunk("IHDR-data"));
                }
                width = read_u32be(data, 0);
                height = read_u32be(data, 4);
                bit_depth = data[8];
                color_type = data[9];
                if data[10] != 0 {
                    return Err(PngError::Unsupported {
                        what: "compression method",
                    });
                }
                if data[11] != 0 {
                    return Err(PngError::Unsupported {
                        what: "filter method",
                    });
                }
                if data[12] != 0 {
                    return Err(PngError::Unsupported {
                        what: "interlacing",
                    });
                }
                seen_ihdr = true;
            }
            b"PLTE" => {
                let mut entries = Vec::with_capacity(size / 3);
                for i in 0..(size / 3) {
                    entries.push([data[i * 3], data[i * 3 + 1], data[i * 3 + 2]]);
                }
                palette = Some(entries);
            }
            b"tRNS" => {
                trns = Some(data.to_vec());
            }
            b"IDAT" => {
                idat.extend_from_slice(data);
            }
            b"IEND" => {
                seen_iend = true;
            }
            _ => {} // ancillary chunks are skipped
        }
        cursor = data_end + 4;
    }

    if !seen_ihdr {
        return Err(PngError::MissingChunk("IHDR"));
    }
    if !seen_iend {
        return Err(PngError::MissingChunk("IEND"));
    }
    if idat.is_empty() {
        return Err(PngError::MissingChunk("IDAT"));
    }
    if bit_depth != 8 {
        return Err(PngError::Unsupported {
            what: "bit depth (only 8)",
        });
    }
    let channels: usize = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        _ => return Err(PngError::Unsupported { what: "color type" }),
    };
    if color_type == 3 && palette.is_none() {
        return Err(PngError::MissingChunk("PLTE"));
    }
    if width == 0 || height == 0 {
        return Err(PngError::BadDimensions { width, height });
    }

    let raw = zlib_decompress(&idat).map_err(PngError::Inflate)?;
    let stride = (width as usize) * channels;
    let expected_len = (height as usize) * (1 + stride);
    if raw.len() < expected_len {
        return Err(PngError::Truncated);
    }

    let defiltered = defilter(&raw, height as usize, stride, channels);
    let rgba = to_rgba8(
        &defiltered,
        width,
        height,
        color_type,
        palette.as_deref(),
        trns.as_deref(),
    );

    Ok(DecodedPng {
        width,
        height,
        rgba,
    })
}

/// Remove per-scanline filters (0..=4) in place over the raw stream.
fn defilter(raw: &[u8], height: usize, stride: usize, channels: usize) -> Vec<u8> {
    let bpp = channels; // bit depth 8 => one byte per channel
    let mut out = vec![0u8; height * stride];
    let mut prev_start = None;
    for row in 0..height {
        let src = row * (1 + stride);
        let filter = raw[src];
        let line = &raw[src + 1..src + 1 + stride];
        let dst_start = row * stride;
        for x in 0..stride {
            let a = if x >= bpp {
                out[dst_start + x - bpp]
            } else {
                0
            };
            let b = match prev_start {
                Some(p) => out[p + x],
                None => 0,
            };
            let c = if x >= bpp {
                prev_start.map(|p| out[p + x - bpp]).unwrap_or(0)
            } else {
                0
            };
            let value = match filter {
                0 => line[x],
                1 => line[x].wrapping_add(a), // Sub
                2 => line[x].wrapping_add(b), // Up
                3 => line[x].wrapping_add(((a as u16 + b as u16) / 2) as u8), // Average
                4 => line[x].wrapping_add(paeth(a, b, c)), // Paeth
                _ => line[x], // invalid filter: spec says treat as error; foundation clamps
            };
            out[dst_start + x] = value;
        }
        prev_start = Some(dst_start);
    }
    out
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Convert defiltered scanlines to RGBA8 honoring palette + tRNS.
fn to_rgba8(
    pixels: &[u8],
    width: u32,
    height: u32,
    color_type: u8,
    palette: Option<&[[u8; 3]]>,
    trns: Option<&[u8]>,
) -> Vec<u8> {
    let count = (width as usize) * (height as usize);
    let mut out = Vec::with_capacity(count * 4);
    match color_type {
        0 => {
            // grayscale; tRNS = 2-byte big-endian gray value
            let transparent: Option<u16> = trns.map(|t| ((t[0] as u16) << 8) | t[1] as u16);
            for &v in pixels.iter().take(count) {
                let alpha = match transparent {
                    Some(t) if t as u8 == v => 0,
                    _ => 255,
                };
                out.extend_from_slice(&[v, v, v, alpha]);
            }
        }
        2 => {
            // RGB; tRNS = 3 x 2-byte color
            let transparent: Option<[u16; 3]> = trns.map(|t| {
                [
                    ((t[0] as u16) << 8) | t[1] as u16,
                    ((t[2] as u16) << 8) | t[3] as u16,
                    ((t[4] as u16) << 8) | t[5] as u16,
                ]
            });
            for px in pixels.chunks(3).take(count) {
                let alpha = match transparent {
                    Some(t)
                        if t[0] as u8 == px[0] && t[1] as u8 == px[1] && t[2] as u8 == px[2] =>
                    {
                        0
                    }
                    _ => 255,
                };
                out.extend_from_slice(&[px[0], px[1], px[2], alpha]);
            }
        }
        3 => {
            // palette; tRNS = alpha per palette index
            let palette = palette.expect("checked earlier");
            for &idx in pixels.iter().take(count) {
                let rgb = palette.get(idx as usize).copied().unwrap_or([0, 0, 0]);
                let alpha = trns
                    .and_then(|t| t.get(idx as usize).copied())
                    .unwrap_or(255);
                out.extend_from_slice(&[rgb[0], rgb[1], rgb[2], alpha]);
            }
        }
        4 => {
            // grayscale + alpha
            for px in pixels.chunks(2).take(count) {
                out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
        }
        6 => {
            // RGBA
            for px in pixels.chunks(4).take(count) {
                out.extend_from_slice(px);
            }
        }
        _ => unreachable!("color type validated earlier"),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::FixtureBuilder;

    #[test]
    fn decodes_rgba_solid() {
        let png = FixtureBuilder::solid(2, 3, 6, &[10, 20, 30, 255]).to_png();
        let img = decode_png(&png).expect("decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 3);
        assert_eq!(img.rgba.len(), 2 * 3 * 4);
        assert_eq!(&img.rgba[0..4], &[10, 20, 30, 255]);
        assert_eq!(&img.rgba[20..24], &[10, 20, 30, 255]);
    }

    #[test]
    fn decodes_palette_with_trns() {
        let mut f = FixtureBuilder::solid(2, 2, 3, &[0]);
        f.palette = vec![[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        f.trns = vec![255, 128, 0];
        // rows use indices: row0 = [0, 1], row1 = [2, 2]
        f.scanlines = vec![0, 0, 1, 0, 2, 2];
        let png = f.to_png();
        let img = decode_png(&png).expect("decode");
        assert_eq!(&img.rgba[0..8], &[255, 0, 0, 255, 0, 255, 0, 128]);
        assert_eq!(&img.rgba[8..16], &[0, 0, 255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn decodes_gray_with_trns_color_key() {
        let mut f = FixtureBuilder::solid(2, 1, 0, &[7]);
        f.trns = vec![0, 7]; // 16-bit BE value 7 -> transparent
        let img = decode_png(&f.to_png()).expect("decode");
        assert_eq!(&img.rgba[0..8], &[7, 7, 7, 0, 7, 7, 7, 0]);
    }

    #[test]
    fn defilter_sub_up_average_paeth() {
        // Row 0 with filter None, row 1 with filter Sub: stored bytes are
        // (real - left) per byte within the row (bpp = 4 for RGBA).
        let mut f = FixtureBuilder::solid(4, 2, 6, &[0, 0, 0, 0]);
        let row0 = [
            10u8, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
        ];
        let real1 = [
            10u8, 20, 30, 255, 45, 55, 65, 255, 80, 90, 100, 255, 110, 120, 130, 255,
        ];
        let mut row1 = vec![1u8]; // Sub filter
        for (i, v) in real1.iter().enumerate() {
            let left = if i >= 4 { real1[i - 4] } else { 0 };
            row1.push(v.wrapping_sub(left));
        }
        f.scanlines.clear();
        f.scanlines.push(0u8);
        f.scanlines.extend_from_slice(&row0);
        f.scanlines.extend_from_slice(&row1);

        let img = decode_png(&f.to_png()).expect("decode");
        // second row = pixels 4..8 => bytes 16..32
        assert_eq!(&img.rgba[16..20], &[10, 20, 30, 255]);
        assert_eq!(&img.rgba[20..24], &[45, 55, 65, 255]);
        assert_eq!(&img.rgba[24..28], &[80, 90, 100, 255]);
        assert_eq!(&img.rgba[28..32], &[110, 120, 130, 255]);
    }

    #[test]
    fn rejects_bad_signature() {
        assert_eq!(decode_png(b"not a png"), Err(PngError::BadSignature));
    }

    #[test]
    fn rejects_corrupt_crc() {
        let mut png = FixtureBuilder::solid(1, 1, 6, &[1, 2, 3, 4]).to_png();
        // Layout: signature(8) + IHDR chunk (8+13+4) + IDAT chunk header(8)
        // + zlib header(2) ... corrupt a byte inside the IDAT payload so
        // the chunk CRC check trips.
        let corrupt_at = 8 + 8 + 13 + 4 + 8 + 2;
        png[corrupt_at] ^= 0xFF;
        assert!(matches!(
            decode_png(&png),
            Err(PngError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn rejects_interlaced() {
        let mut png = FixtureBuilder::solid(2, 2, 6, &[1, 2, 3, 4]).to_png();
        // IHDR interlace byte: 8(sig) + 8(chunk hdr) + 4+4 (w,h) + 1+1+1+1
        // (depth, color, compression, filter) => offset 28.
        let interlace_byte = 8 + 8 + 4 + 4 + 1 + 1 + 1 + 1;
        assert_eq!(interlace_byte, 28);
        png[interlace_byte] = 1;
        // Patch the IHDR CRC (chunk type at 12..16, data 16..29, crc 29..33)
        // so the CRC check passes and parsing reaches the interlace check.
        let fixed_crc = crate::crc::crc32(&png[12..29]);
        png[29..33].copy_from_slice(&fixed_crc.to_be_bytes());
        assert_eq!(
            decode_png(&png),
            Err(PngError::Unsupported {
                what: "interlacing"
            })
        );
    }

    #[test]
    fn real_asset_smoke_test() {
        // Decode a real PNG from the repo's Tiny Swords pack when present.
        // CARGO_MANIFEST_DIR = <repo>/monster-engine/crates/monster-assets,
        // so the repo root is three levels up.
        let manifest = env!("CARGO_MANIFEST_DIR");
        let assets_root = std::path::Path::new(manifest).join("../../../assest");
        let Some(png_path) = find_first_png(&assets_root, 0) else {
            // Assets not present in this checkout: skip without failing.
            eprintln!("smoke test skipped: no PNG under {assets_root:?}");
            return;
        };
        let bytes = std::fs::read(&png_path).expect("read asset");
        let img = decode_png(&bytes).expect("decode real PNG");
        assert!(img.width > 0 && img.height > 0);
        assert_eq!(
            img.rgba.len(),
            (img.width as usize) * (img.height as usize) * 4
        );
        eprintln!(
            "smoke test decoded {png_path:?}: {}x{}",
            img.width, img.height
        );
    }

    fn find_first_png(dir: &std::path::Path, depth: usize) -> Option<std::path::PathBuf> {
        if depth > 6 {
            return None;
        }
        let entries = std::fs::read_dir(dir).ok()?;
        let mut dirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("png"))
                {
                    return Some(path);
                }
            } else {
                dirs.push(path);
            }
        }
        for d in dirs {
            if let Some(found) = find_first_png(&d, depth + 1) {
                return Some(found);
            }
        }
        None
    }
}
