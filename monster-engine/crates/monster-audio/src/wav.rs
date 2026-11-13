//! RIFF/WAVE PCM decoder producing [`AudioClip`]s.

use std::fmt;

use monster_core::audio::AudioClip;

/// WAV decode failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WavError {
    /// Missing/invalid "RIFF"/"WAVE" markers.
    NotRiff,
    /// `fmt ` chunk missing or truncated.
    MissingFmt,
    /// `data` chunk missing or truncated.
    MissingData,
    /// Only PCM-16 is supported.
    UnsupportedFormat { format: u16, bits: u16 },
    /// Chunk sizes exceed the available bytes.
    Truncated {
        chunk: String,
        needed: usize,
        have: usize,
    },
}

impl fmt::Display for WavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WavError::NotRiff => write!(f, "not a RIFF/WAVE file"),
            WavError::MissingFmt => write!(f, "missing or invalid fmt chunk"),
            WavError::MissingData => write!(f, "missing or empty data chunk"),
            WavError::UnsupportedFormat { format, bits } => {
                write!(f, "unsupported WAV format {format} with {bits}-bit samples")
            }
            WavError::Truncated {
                chunk,
                needed,
                have,
            } => {
                write!(
                    f,
                    "truncated chunk '{chunk}': need {needed} bytes, have {have}"
                )
            }
        }
    }
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

/// Decode a RIFF/WAVE file (PCM, 8/16/24-bit) into a mono 16-bit clip.
///
/// - Stereo/multi-channel inputs are mixed down to mono by averaging.
/// - 8-bit PCM is unsigned; 16-bit is little-endian signed; 24-bit is
///   little-endian signed, scaled down to 16-bit.
pub fn decode_wav(bytes: &[u8]) -> Result<AudioClip, WavError> {
    // RIFF header: "RIFF" + size + "WAVE"
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavError::NotRiff);
    }

    // Walk chunks.
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut bits = 0u16;
    let mut audio_format = 0u16;
    let mut data: Option<&[u8]> = None;

    let mut cursor = 12usize;
    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size = read_u32le(bytes, cursor + 4) as usize;
        let body_start = cursor + 8;
        let body_end = body_start.checked_add(size).ok_or(WavError::MissingData)?;
        if body_end > bytes.len() {
            return Err(WavError::Truncated {
                chunk: String::from_utf8_lossy(id).into_owned(),
                needed: size,
                have: bytes.len() - body_start,
            });
        }
        let body = &bytes[body_start..body_end];
        match id {
            b"fmt " => {
                if body.len() < 16 {
                    return Err(WavError::MissingFmt);
                }
                audio_format = read_u16le(body, 0);
                channels = read_u16le(body, 2);
                sample_rate = read_u32le(body, 4);
                bits = read_u16le(body, 14);
            }
            b"data" => {
                data = Some(body);
            }
            _ => {}
        }
        // Chunks are word-aligned.
        cursor = body_end + (size % 2);
    }

    let Some(data) = data else {
        return Err(WavError::MissingData);
    };
    if channels == 0 {
        return Err(WavError::MissingFmt);
    }
    if !(audio_format == 1 || audio_format == 0xFFFE) {
        return Err(WavError::UnsupportedFormat {
            format: audio_format,
            bits,
        });
    }

    // Extract per-sample values as i16.
    let bytes_per_sample = (bits / 8) as usize;
    let samples_i16: Vec<i16> = match bits {
        16 => {
            let count = data.len() / 2;
            (0..count)
                .map(|i| i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]))
                .collect()
        }
        8 => data
            .iter()
            .map(|b| ((*b as i16 - 128) << 8) as i16)
            .collect(),
        24 => {
            let count = data.len() / 3;
            (0..count)
                .map(|i| {
                    let raw = ((data[i * 3 + 2] as i32) << 24
                        | (data[i * 3 + 1] as i32) << 16
                        | (data[i * 3] as i32) << 8)
                        >> 8; // sign-extend 24 -> 32
                    (raw >> 8) as i16
                })
                .collect()
        }
        _ => {
            return Err(WavError::UnsupportedFormat {
                format: audio_format,
                bits,
            })
        }
    };

    if bytes_per_sample == 0 {
        return Err(WavError::UnsupportedFormat {
            format: audio_format,
            bits,
        });
    }

    // Mix down to mono.
    let ch = channels as usize;
    let mono: Vec<i16> = if ch <= 1 {
        samples_i16
    } else {
        samples_i16
            .chunks(ch)
            .map(|frame| {
                let sum: i32 = frame.iter().map(|s| *s as i32).sum();
                (sum / ch as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16
            })
            .collect()
    };

    Ok(AudioClip::new(sample_rate, mono))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal PCM-16 WAV file in-memory.
    fn build_wav(channels: u16, sample_rate: u32, samples: &[i16]) -> Vec<u8> {
        let mut out = Vec::new();
        let data_bytes = samples.len() * 2;
        let fmt_chunk: [u8; 24] = {
            let mut c = Vec::new();
            c.extend_from_slice(b"fmt ");
            c.extend_from_slice(&16u32.to_le_bytes());
            c.extend_from_slice(&1u16.to_le_bytes()); // PCM
            c.extend_from_slice(&channels.to_le_bytes());
            c.extend_from_slice(&sample_rate.to_le_bytes());
            let byte_rate = sample_rate * channels as u32 * 2;
            c.extend_from_slice(&byte_rate.to_le_bytes());
            c.extend_from_slice(&(channels * 2).to_le_bytes()); // block align
            c.extend_from_slice(&16u16.to_le_bytes()); // bits
            let mut arr = [0u8; 24];
            arr.copy_from_slice(&c);
            arr
        };
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&((4 + 24 + 8 + data_bytes) as u32).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(&fmt_chunk);
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data_bytes as u32).to_le_bytes());
        for s in samples {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }

    #[test]
    fn decodes_mono_pcm16() {
        let samples = vec![100i16, -200, 300, -400];
        let bytes = build_wav(1, 22050, &samples);
        let clip = decode_wav(&bytes).expect("decode");
        assert_eq!(clip.sample_rate, 22050);
        assert_eq!(clip.samples, samples);
    }

    #[test]
    fn stereo_mixed_down_to_mono() {
        // frame (L, R): (1000, -500) -> 250 ; (-1000, 500) -> -250
        let frames = [1000i16, -500, -1000, 500];
        let bytes = build_wav(2, 44100, &frames);
        let clip = decode_wav(&bytes).expect("decode");
        assert_eq!(clip.sample_rate, 44100);
        assert_eq!(clip.samples, vec![250, -250]);
    }

    #[test]
    fn rejects_non_riff_and_truncated() {
        assert_eq!(decode_wav(b"NOPE...."), Err(WavError::NotRiff));
        let mut bytes = build_wav(1, 22050, &[1, 2, 3]);
        // file is 46 bytes: 12 header + 24 fmt + 10 data; cut the last byte
        // so the data chunk header parses but its body is short
        bytes.truncate(45);
        assert!(matches!(
            decode_wav(&bytes),
            Err(WavError::Truncated { .. })
        ));
        let no_data = {
            let mut b = build_wav(1, 22050, &[1]);
            // rename data chunk to "junk"
            let pos = b.windows(4).position(|w| w == b"data").unwrap();
            b[pos..pos + 4].copy_from_slice(b"junk");
            b
        };
        assert_eq!(decode_wav(&no_data), Err(WavError::MissingData));
    }

    #[test]
    fn skips_extra_chunks_and_alignment() {
        // Insert an odd-sized chunk between fmt and data; the reader must
        // skip the alignment pad byte correctly.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        let body_len = 24 + (8 + 3 + 1) + (8 + 4); // fmt + odd chunk + data(2 samples)
        bytes.extend_from_slice(&(4 + body_len as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        // fmt
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&16000u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        // odd-sized metadata chunk "LIST" with 3 bytes + pad
        bytes.extend_from_slice(b"LIST");
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(b"abc");
        bytes.push(0); // alignment pad
                       // data: 2 samples
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&7i16.to_le_bytes());
        bytes.extend_from_slice(&(-7i16).to_le_bytes());

        let clip = decode_wav(&bytes).expect("decode");
        assert_eq!(clip.sample_rate, 8000);
        assert_eq!(clip.samples, vec![7, -7]);
    }
}
