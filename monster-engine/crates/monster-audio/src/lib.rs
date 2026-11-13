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
pub use wav::{decode_wav, WavError};
