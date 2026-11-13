//! DEFLATE (RFC 1951) decompressor and the zlib (RFC 1950) wrapper.
//!
//! Supports all three block types — stored, fixed Huffman and dynamic
//! Huffman — which is everything real PNG files use.

use std::fmt;

use crate::crc::adler32;

/// Inflate failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    OutOfData,
    InvalidBlockType,
    InvalidStoredBlock,
    InvalidCodeLengths,
    InvalidHuffmanCode,
    InvalidBackReference,
    BadZlibHeader,
    AdlerMismatch { expected: u32, got: u32 },
}

impl fmt::Display for InflateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InflateError::OutOfData => write!(f, "unexpected end of deflate stream"),
            InflateError::InvalidBlockType => write!(f, "invalid block type"),
            InflateError::InvalidStoredBlock => write!(f, "invalid stored block"),
            InflateError::InvalidCodeLengths => write!(f, "over-subscribed code lengths"),
            InflateError::InvalidHuffmanCode => write!(f, "invalid huffman code"),
            InflateError::InvalidBackReference => write!(f, "back-reference before stream start"),
            InflateError::BadZlibHeader => write!(f, "bad zlib header"),
            InflateError::AdlerMismatch { expected, got } => {
                write!(
                    f,
                    "adler32 mismatch: expected {expected:#010x}, got {got:#010x}"
                )
            }
        }
    }
}

/// LSB-first bit reader over a byte slice.
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buf: u32,
    bit_count: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> BitReader<'a> {
        BitReader {
            data,
            pos: 0,
            bit_buf: 0,
            bit_count: 0,
        }
    }

    fn read_bits(&mut self, count: u32) -> Result<u32, InflateError> {
        debug_assert!(count <= 16);
        while self.bit_count < count {
            if self.pos >= self.data.len() {
                return Err(InflateError::OutOfData);
            }
            self.bit_buf |= (self.data[self.pos] as u32) << self.bit_count;
            self.pos += 1;
            self.bit_count += 8;
        }
        let value = self.bit_buf & ((1u32 << count) - 1);
        self.bit_buf >>= count;
        self.bit_count -= count;
        Ok(value)
    }

    /// Discard bits up to the next byte boundary.
    fn align_to_byte(&mut self) {
        let drop = self.bit_count % 8;
        self.bit_buf >>= drop;
        self.bit_count -= drop;
    }
}

/// Canonical Huffman decoder built from code lengths.
struct Huffman {
    /// Number of codes per bit length (index = length).
    counts: [u16; 16],
    /// Symbols ordered by (length, symbol value).
    symbols: Vec<u16>,
}

impl Huffman {
    fn from_lengths(lengths: &[u8]) -> Result<Huffman, InflateError> {
        let mut counts = [0u16; 16];
        for &len in lengths {
            if len > 15 {
                return Err(InflateError::InvalidCodeLengths);
            }
            counts[len as usize] += 1;
        }
        counts[0] = 0;

        // Over-subscription check (Kraft inequality): a complete tree gives
        // exactly 2^16 under this accumulation; anything beyond is invalid.
        let mut total: u32 = 0;
        for len in 1..16 {
            total = (total + counts[len] as u32) << 1;
        }
        if total > (1 << 16) {
            return Err(InflateError::InvalidCodeLengths);
        }

        let mut symbols = Vec::with_capacity(lengths.len());
        for len in 1..16usize {
            for (sym, &l) in lengths.iter().enumerate() {
                if l as usize == len {
                    symbols.push(sym as u16);
                }
            }
        }
        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, br: &mut BitReader<'_>) -> Result<u16, InflateError> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for len in 1..16usize {
            code |= br.read_bits(1)? as i32;
            let count = self.counts[len] as i32;
            if code < first + count {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(InflateError::InvalidHuffmanCode)
    }
}

/// Fixed (pre-defined) literal/length code lengths, RFC 1951 §3.2.6.
fn fixed_literal_lengths() -> Vec<u8> {
    let mut lengths = vec![0u8; 288];
    for (i, len) in lengths.iter_mut().enumerate() {
        *len = match i {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    lengths
}

/// Fixed distance code lengths: 30 symbols of 5 bits (plus 2 unused).
fn fixed_distance_lengths() -> Vec<u8> {
    vec![5u8; 30]
}

// Length codes 257..285: base length and extra bits (RFC 1951 §3.2.5).
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

// Distance codes 0..29: base distance and extra bits.
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// Code-length alphabet order (RFC 1951 §3.2.7).
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Inflate a raw DEFLATE stream.
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, InflateError> {
    let mut br = BitReader::new(data);
    let mut out = Vec::with_capacity(data.len() * 4);
    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        match btype {
            0 => inflate_stored(&mut br, &mut out)?,
            1 => {
                let lit = Huffman::from_lengths(&fixed_literal_lengths())?;
                let dist = Huffman::from_lengths(&fixed_distance_lengths())?;
                inflate_block(&mut br, &mut out, &lit, &dist)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut br)?;
                inflate_block(&mut br, &mut out, &lit, &dist)?;
            }
            _ => return Err(InflateError::InvalidBlockType),
        }
        if bfinal == 1 {
            break;
        }
    }
    Ok(out)
}

fn inflate_stored(br: &mut BitReader<'_>, out: &mut Vec<u8>) -> Result<(), InflateError> {
    br.align_to_byte();
    let len = br.read_bits(16)? as usize;
    let nlen = br.read_bits(16)? as usize;
    if len ^ 0xFFFF != nlen {
        return Err(InflateError::InvalidStoredBlock);
    }
    // bytes left in the bit buffer are whole bytes (we just aligned)
    for _ in 0..len {
        out.push(br.read_bits(8)? as u8);
    }
    Ok(())
}

fn dynamic_tables(br: &mut BitReader<'_>) -> Result<(Huffman, Huffman), InflateError> {
    let hlit = br.read_bits(5)? as usize + 257;
    let hdist = br.read_bits(5)? as usize + 1;
    let hclen = br.read_bits(4)? as usize + 4;

    let mut cl_lengths = [0u8; 19];
    for i in 0..hclen {
        cl_lengths[CODE_LENGTH_ORDER[i]] = br.read_bits(3)? as u8;
    }
    let cl_tree = Huffman::from_lengths(&cl_lengths)?;

    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0usize;
    while i < lengths.len() {
        let sym = cl_tree.decode(br)?;
        match sym {
            0..=15 => {
                lengths[i] = sym as u8;
                i += 1;
            }
            16 => {
                if i == 0 {
                    return Err(InflateError::InvalidCodeLengths);
                }
                let repeat = 3 + br.read_bits(2)? as usize;
                let prev = lengths[i - 1];
                for _ in 0..repeat {
                    if i >= lengths.len() {
                        return Err(InflateError::InvalidCodeLengths);
                    }
                    lengths[i] = prev;
                    i += 1;
                }
            }
            17 => {
                let repeat = 3 + br.read_bits(3)? as usize;
                for _ in 0..repeat {
                    if i >= lengths.len() {
                        return Err(InflateError::InvalidCodeLengths);
                    }
                    lengths[i] = 0;
                    i += 1;
                }
            }
            18 => {
                let repeat = 11 + br.read_bits(7)? as usize;
                for _ in 0..repeat {
                    if i >= lengths.len() {
                        return Err(InflateError::InvalidCodeLengths);
                    }
                    lengths[i] = 0;
                    i += 1;
                }
            }
            _ => return Err(InflateError::InvalidCodeLengths),
        }
    }
    let lit = Huffman::from_lengths(&lengths[..hlit])?;
    let dist = Huffman::from_lengths(&lengths[hlit..])?;
    Ok((lit, dist))
}

fn inflate_block(
    br: &mut BitReader<'_>,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
) -> Result<(), InflateError> {
    loop {
        let sym = lit.decode(br)?;
        #[cfg(feature = "inflate-trace")]
        eprintln!("sym={sym} out_len={}", out.len());
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let idx = (sym - 257) as usize;
                let length =
                    LENGTH_BASE[idx] as usize + br.read_bits(LENGTH_EXTRA[idx] as u32)? as usize;
                let dsym = dist.decode(br)? as usize;
                if dsym >= DIST_BASE.len() {
                    return Err(InflateError::InvalidBackReference);
                }
                let distance =
                    DIST_BASE[dsym] as usize + br.read_bits(DIST_EXTRA[dsym] as u32)? as usize;
                if distance > out.len() || distance == 0 {
                    return Err(InflateError::InvalidBackReference);
                }
                // Copy byte-by-byte: overlapping references repeat within window.
                let start = out.len() - distance;
                for k in 0..length {
                    let byte = out[start + k];
                    out.push(byte);
                }
            }
            _ => return Err(InflateError::InvalidHuffmanCode),
        }
    }
}

/// Decompress a zlib (RFC 1950) stream: 2-byte header + deflate + adler32.
pub fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, InflateError> {
    if data.len() < 6 {
        return Err(InflateError::BadZlibHeader);
    }
    let cmf = data[0];
    let flg = data[1];
    if cmf & 0x0F != 8 {
        return Err(InflateError::BadZlibHeader);
    }
    if ((cmf as u32) * 256 + flg as u32) % 31 != 0 {
        return Err(InflateError::BadZlibHeader);
    }
    if flg & 0x20 != 0 {
        return Err(InflateError::BadZlibHeader); // preset dictionary unsupported
    }
    let body = &data[2..];
    let out = inflate(body)?;
    // The zlib stream ends with a 4-byte big-endian adler32 of the payload.
    // `inflate` stops at the final block, so the trailing 4 bytes are it.
    let tail = &data[data.len() - 4..];
    let expected = u32::from_be_bytes([tail[0], tail[1], tail[2], tail[3]]);
    let got = adler32(&out);
    if expected != got {
        return Err(InflateError::AdlerMismatch { expected, got });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Append a stored-block zlib stream for `payload` (test helper).
    fn zlib_stored(payload: &[u8], final_block: bool) -> Vec<u8> {
        let mut out = vec![0x78u8, 0x01u8]; // CMF/FLG with valid FCHECK
        let bfinal = if final_block { 1u8 } else { 0u8 };
        out.push(bfinal); // bfinal bit | btype=00 (rest of byte discarded)
        out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
        out.extend_from_slice(payload);
        out.extend_from_slice(&adler32(payload).to_be_bytes());
        out
    }

    #[test]
    fn stored_block_roundtrip() {
        let stream = zlib_stored(b"hello, world!", true);
        let out = zlib_decompress(&stream).expect("decompress");
        assert_eq!(out, b"hello, world!");
    }

    #[test]
    fn two_stored_blocks() {
        let mut out = vec![0x78u8, 0x01u8];
        for (payload, last) in [(b"abc".as_slice(), false), (b"defg".as_slice(), true)] {
            out.push(if last { 1u8 } else { 0u8 });
            out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
            out.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
            out.extend_from_slice(payload);
        }
        out.extend_from_slice(&adler32(b"abcdefg").to_be_bytes());
        let out = zlib_decompress(&out).expect("decompress");
        assert_eq!(out, b"abcdefg");
    }

    #[test]
    fn fixed_huffman_literals() {
        // Craft bits for literals 'A'(65) 'B'(66) then EOB(256) in the fixed
        // tree: codes are 8-bit (0x30 + sym) for 0..=143, EOB is 7-bit zero.
        // Huffman codes go into the stream MSB-first (RFC 1951 §3.1.1),
        // while the byte/bit container itself is LSB-first.
        let mut bits: Vec<bool> = Vec::new();
        let mut push_code = |bits: &mut Vec<bool>, code: u32, len: u32| {
            for i in (0..len).rev() {
                bits.push((code >> i) & 1 == 1); // MSB of the code first
            }
        };
        push_code(&mut bits, 0x30 + 65, 8);
        push_code(&mut bits, 0x30 + 66, 8);
        push_code(&mut bits, 0, 7); // EOB
        let stream = pack_fixed_block(bits);
        let stream = attach_adler(stream, b"AB");
        let out = zlib_decompress(&stream).expect("decompress");
        assert_eq!(out, b"AB");
    }

    #[test]
    fn fixed_huffman_back_reference() {
        // Literal 'X' (88: code 0x30+88), then length sym 257 (len 3, 7-bit
        // code = 1), distance sym 0 (dist 1, 5-bit code = 0), then EOB.
        // Output: "XXXX".
        let mut bits: Vec<bool> = Vec::new();
        let mut push_code = |bits: &mut Vec<bool>, code: u32, len: u32| {
            for i in (0..len).rev() {
                bits.push((code >> i) & 1 == 1);
            }
        };
        push_code(&mut bits, 0x30 + 88, 8); // 'X'
        push_code(&mut bits, 1, 7); // length 257 -> 3
        push_code(&mut bits, 0, 5); // distance 0 -> 1
        push_code(&mut bits, 0, 7); // EOB
        let stream = attach_adler(pack_fixed_block(bits), b"XXXX");
        let out = zlib_decompress(&stream).expect("decompress");
        assert_eq!(out, b"XXXX");
    }

    /// zlib header + one fixed-Huffman block carrying `bits`.
    ///
    /// The body is one continuous LSB-first bitstream: bfinal=1, btype=01
    /// (two bits), then the code bits — no byte boundary after the header.
    fn pack_fixed_block(bits: Vec<bool>) -> Vec<u8> {
        let mut body_bits: Vec<bool> = vec![true, true, false]; // bfinal=1, btype=01
        body_bits.extend(bits);
        let mut stream = vec![0x78u8, 0x01u8];
        let mut byte = 0u8;
        let mut shift = 0u32;
        for bit in body_bits {
            if bit {
                byte |= 1 << shift;
            }
            shift += 1;
            if shift == 8 {
                stream.push(byte);
                byte = 0;
                shift = 0;
            }
        }
        if shift > 0 {
            stream.push(byte);
        }
        stream
    }

    fn attach_adler(mut stream: Vec<u8>, payload: &[u8]) -> Vec<u8> {
        stream.extend_from_slice(&adler32(payload).to_be_bytes());
        stream
    }

    #[test]
    fn rejects_bad_zlib_and_adler() {
        assert_eq!(zlib_decompress(&[]), Err(InflateError::BadZlibHeader));
        assert_eq!(
            zlib_decompress(&[0x79, 0x01, 1, 2, 3, 4, 5, 6]),
            Err(InflateError::BadZlibHeader)
        );
        // valid header, corrupt adler
        let mut stream = zlib_stored(b"hi", true);
        let last = stream.len() - 1;
        stream[last] ^= 0xFF;
        assert!(matches!(
            zlib_decompress(&stream),
            Err(InflateError::AdlerMismatch { .. })
        ));
    }

    #[test]
    fn dynamic_via_real_png_assets() {
        // Real-world PNGs (Tiny Swords) use dynamic Huffman blocks; that
        // path is exercised in png::tests via the actual asset files.
    }
}
