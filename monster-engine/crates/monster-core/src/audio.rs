//! Audio contracts: clip data and the playback backend trait.
//!
//! Data (decode) and playback (device I/O) are deliberately separated so a
//! real playback backend (waveOut/DirectSound via FFI, or WebAudio when
//! targeting the browser) can be added without touching clip loaders or
//! game code.

/// PCM audio in mono 16-bit signed format — the engine's clip currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioClip {
    pub sample_rate: u32,
    /// Mono PCM-16 samples.
    pub samples: Vec<i16>,
}

impl AudioClip {
    pub fn new(sample_rate: u32, samples: Vec<i16>) -> AudioClip {
        AudioClip {
            sample_rate,
            samples,
        }
    }

    /// Duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f32 / self.sample_rate as f32
        }
    }

    /// Generate a sine-wave clip (useful for tests and demo SFX).
    pub fn sine(sample_rate: u32, freq_hz: f32, seconds: f32, amplitude: i16) -> AudioClip {
        let count = ((sample_rate as f32) * seconds).round() as usize;
        let mut samples = Vec::with_capacity(count);
        for i in 0..count {
            let t = i as f32 / sample_rate as f32;
            let v = (t * freq_hz * std::f32::consts::TAU).sin() * amplitude as f32;
            samples.push(v.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
        }
        AudioClip::new(sample_rate, samples)
    }
}

/// Handle to an in-flight playback.
pub type AudioPlaybackId = u64;

/// Playback backend interface.
///
/// `play_clip` copies the data it needs (backends own their transport);
/// returns a unique id the caller can use to stop that playback.
pub trait AudioBackend {
    /// Begin playing `clip` at `volume` (0.0..1.0), optionally looping.
    fn play_clip(&mut self, clip: &AudioClip, volume: f32, looped: bool) -> AudioPlaybackId;

    /// Stop a specific playback.
    fn stop(&mut self, id: AudioPlaybackId);

    /// Stop everything currently playing.
    fn stop_all(&mut self);

    /// Number of currently active playbacks (for tests/diagnostics).
    fn active_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_clip_shape_and_duration() {
        let clip = AudioClip::sine(44100, 440.0, 0.5, 10000);
        assert_eq!(clip.sample_rate, 44100);
        assert_eq!(clip.samples.len(), 22050);
        assert!(monster_math::approx_eq(clip.duration_secs(), 0.5, 1e-6));
        // first sample of a sine starting at phase 0 is ~0
        assert!(clip.samples[0].abs() < 100);
        // peak amplitude within bounds
        assert!(clip.samples.iter().all(|s| s.abs() <= 10000));
    }

    #[test]
    fn zero_rate_duration_is_zero() {
        let clip = AudioClip::new(0, vec![1, 2, 3]);
        assert_eq!(clip.duration_secs(), 0.0);
    }
}
