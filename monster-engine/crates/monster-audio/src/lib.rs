//! Audio for `monster-engine`.
//!
//! Two halves, cleanly split so playback backends are pluggable:
//! - [`wav`]: decode RIFF/WAVE PCM files into the engine's [`AudioClip`].
//! - [`NullAudioBackend`]: a real [`AudioBackend`] implementation that
//!   plays nothing (mute) — the default in headless runs and the reference
//!   for future device backends (waveOut/DirectSound via FFI, WebAudio on
//!   the web).
//!
//! Adding a hardware playback backend = implementing `AudioBackend` in a
//! new module; clip decoding and game code stay untouched.

pub mod backend;
pub mod wav;

pub use backend::NullAudioBackend;
use monster_core::audio::AudioClip;
pub use wav::{decode_wav, WavError};

/// Load and decode a WAV file from disk into an [`AudioClip`].
///
/// # Errors
/// - [`WavError::Io`] when the file cannot be read.
/// - Decode errors from [`decode_wav`].
pub fn load_wav_file(path: &std::path::Path) -> Result<AudioClip, WavError> {
    let bytes = std::fs::read(path).map_err(|e| WavError::Io(format!("{path:?}: {e}")))?;
    decode_wav(&bytes)
}
