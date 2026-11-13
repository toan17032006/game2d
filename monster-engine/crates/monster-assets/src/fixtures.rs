//! Shared PNG test fixtures (test-only): a minimal PNG encoder using zlib
//! stored blocks, CRC'd chunks — the round trip target for the decoder.

use crate::crc::{adler32, crc32};
use crate::png::SIGNATURE;

/// Builds small PNG files for decoder tests.
pub struct FixtureBuilder {
    pub width: u32,
    pub height: u32,
    pub color_type: u8,
    pub palette: Vec<[u8; 3]>,
    pub trns: Vec<u8>,
    /// Pre-built scanlines: per row `[filter_byte, pixels...]`.
    pub scanlines: Vec<u8>,
}

impl FixtureBuilder {
    /// A solid-color image with filter-0 scanlines.
    pub fn solid(width: u32, height: u32, color_type: u8, pixel: &[u8]) -> FixtureBuilder {
        let channels = match color_type {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => panic!("unsupported color type in fixture"),
        };
        let mut scanlines = Vec::new();
        for _ in 0..height {
            scanlines.push(0u8);
            for _ in 0..width {
                scanlines.extend_from_slice(&pixel[..channels]);
            }
        }
        FixtureBuilder {
            width,
            height,
            color_type,
            palette: Vec::new(),
            trns: Vec::new(),
            scanlines,
        }
    }

    /// Serialize to PNG bytes (zlib stored blocks, valid CRCs).
    pub fn to_png(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&SIGNATURE);

        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&self.width.to_be_bytes());
        ihdr.extend_from_slice(&self.height.to_be_bytes());
        ihdr.push(8); // bit depth
        ihdr.push(self.color_type);
        ihdr.extend_from_slice(&[0, 0, 0]); // compression, filter, interlace
        push_chunk(&mut out, b"IHDR", &ihdr);

        if self.color_type == 3 {
            let mut plte = Vec::new();
            for rgb in &self.palette {
                plte.extend_from_slice(rgb);
            }
            push_chunk(&mut out, b"PLTE", &plte);
        }
        // tRNS applies to palette AND grayscale/RGB color keys.
        if !self.trns.is_empty() {
            push_chunk(&mut out, b"tRNS", &self.trns);
        }

        // zlib stream: stored-block deflate of the scanlines
        let mut idat = vec![0x78u8, 0x01u8];
        let payload = &self.scanlines;
        const CHUNK: usize = 65535;
        let mut offset = 0usize;
        loop {
            let take = (payload.len() - offset).min(CHUNK);
            let last = offset + take >= payload.len();
            idat.push(if last { 1 } else { 0 });
            idat.extend_from_slice(&(take as u16).to_le_bytes());
            idat.extend_from_slice(&(!(take as u16)).to_le_bytes());
            idat.extend_from_slice(&payload[offset..offset + take]);
            offset += take;
            if last {
                break;
            }
        }
        idat.extend_from_slice(&adler32(payload).to_be_bytes());
        push_chunk(&mut out, b"IDAT", &idat);
        push_chunk(&mut out, b"IEND", &[]);
        out
    }
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}
