//! [`NullAudioBackend`] — mute playback backend.

use monster_core::audio::{AudioBackend, AudioClip, AudioPlaybackId};

/// One tracked playback entry (diagnostics view of what a real backend
/// would be playing).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Playback {
    pub id: AudioPlaybackId,
    pub volume: f32,
    pub looped: bool,
    pub duration_secs: f32,
}

/// A working [`AudioBackend`] that records playbacks and outputs nothing.
///
/// Used for headless runs, tests, and as the default engine backend until a
/// device backend is plugged in. Its bookkeeping mirrors what a real
/// backend must track (id sequence, loop flag, volume, duration), so
/// switching backends later is behavior-compatible.
#[derive(Debug, Default)]
pub struct NullAudioBackend {
    next_id: AudioPlaybackId,
    active: Vec<Playback>,
    master_volume: f32,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            next_id: 1,
            active: Vec::new(),
            master_volume: 1.0,
        }
    }

    /// Master volume multiplier (0.0..1.0).
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Details of a tracked playback (diagnostics/tests).
    pub fn playback(&self, id: AudioPlaybackId) -> Option<&Playback> {
        self.active.iter().find(|p| p.id == id)
    }
}

impl AudioBackend for NullAudioBackend {
    fn play_clip(&mut self, clip: &AudioClip, volume: f32, looped: bool) -> AudioPlaybackId {
        let id = self.next_id;
        self.next_id += 1;
        self.active.push(Playback {
            id,
            volume: volume.clamp(0.0, 1.0),
            looped,
            duration_secs: clip.duration_secs(),
        });
        id
    }

    fn stop(&mut self, id: AudioPlaybackId) {
        self.active.retain(|p| p.id != id);
    }

    fn stop_all(&mut self) {
        self.active.clear();
    }

    fn active_count(&self) -> usize {
        self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_core::audio::AudioClip;

    #[test]
    fn play_stop_cycle() {
        let clip = AudioClip::sine(22050, 440.0, 0.1, 8000);
        let mut backend = NullAudioBackend::new();
        assert_eq!(backend.active_count(), 0);

        let a = backend.play_clip(&clip, 0.8, false);
        let b = backend.play_clip(&clip, 1.0, true);
        assert_ne!(a, b);
        assert_eq!(backend.active_count(), 2);
        assert_eq!(
            backend.playback(b).map(|p| (p.looped, p.duration_secs)),
            Some((true, 0.1))
        );

        backend.stop(a);
        assert_eq!(backend.active_count(), 1);
        backend.stop_all();
        assert_eq!(backend.active_count(), 0);
        backend.stop(a); // stopping an unknown id is a no-op
        assert_eq!(backend.active_count(), 0);
    }

    #[test]
    fn volumes_are_clamped() {
        let clip = AudioClip::new(8000, vec![0; 800]);
        let mut backend = NullAudioBackend::new();
        let id = backend.play_clip(&clip, 4.0, false);
        assert_eq!(backend.playback(id).unwrap().volume, 1.0);
        backend.set_master_volume(2.0);
        assert_eq!(backend.master_volume(), 1.0);
        backend.set_master_volume(-1.0);
        assert_eq!(backend.master_volume(), 0.0);
    }
}
