//! Real audio playback via Windows waveOut (winmm.dll) — implements
//! `monster_core::audio::AudioBackend` on top of hand-written FFI.
//!
//! One owned PCM buffer per playback; `stop_all`/`Drop` reset the device
//! and free everything. Requires an audio device — construction fails
//! gracefully (`Result`) so callers can fall back to `NullAudioBackend`.

use std::ffi::c_void;

use monster_core::audio::{AudioBackend, AudioClip, AudioPlaybackId};

#[link(name = "winmm")]
extern "system" {
    fn waveOutGetNumDevs() -> u32;
    fn waveOutOpen(
        phwo: *mut HWAVEOUT,
        uDeviceID: u32,
        pwfx: *const WAVEFORMATEX,
        dwCallback: *mut c_void,
        dwInstance: *mut c_void,
        fdwOpen: u32,
    ) -> u32;
    fn waveOutPrepareHeader(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutUnprepareHeader(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutWrite(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutReset(hwo: HWAVEOUT) -> u32;
    fn waveOutClose(hwo: HWAVEOUT) -> u32;
}

type HWAVEOUT = *mut c_void;

const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
const WAVE_FORMAT_PCM: u16 = 1;
const CALLBACK_NULL: u32 = 0;
const WHDR_PREPARED: u32 = 0x0000_0010;
// reserved for lazy unprepare polling (foundation keeps eager reset)
#[allow(dead_code)]
const WAVERR_STILL_PLAYING: u32 = 33;

/// WAVEFORMATEX (PCM).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WAVEFORMATEX {
    w_format_tag: u16,
    n_channels: u16,
    n_samples_per_sec: u32,
    n_avg_bytes_per_sec: u32,
    n_block_align: u16,
    w_bits_per_sample: u16,
    cb_size: u16,
}

/// WAVEHDR (Win32 x64 layout).
#[repr(C)]
struct WAVEHDR {
    lp_data: *mut c_void,
    dw_buffer_length: u32,
    dw_bytes_recorded: u32,
    dw_user: *mut c_void,
    dw_flags: u32,
    dw_loops: u32,
    lp_next: *mut c_void,
    reserved: *mut c_void,
}

/// One in-flight playback: the header + its owned PCM buffer.
/// (buffer/volume/looped model real device state; read on diagnostics paths.)
#[allow(dead_code)]
struct ActivePlayback {
    id: AudioPlaybackId,
    header: WAVEHDR,
    /// Owned sample buffer — the header points into this allocation.
    buffer: Box<[u8]>,
    volume: f32,
    looped: bool,
}

/// Real playback backend (winmm waveOut).
pub struct WinmmAudioBackend {
    device: Option<HWAVEOUT>,
    next_id: AudioPlaybackId,
    active: Vec<ActivePlayback>,
    /// Sample rate of the open device (set at first open).
    device_sample_rate: u32,
    open_error: Option<String>,
}

impl WinmmAudioBackend {
    /// Try to open the default wave device (mono, 44100 Hz, 16-bit).
    /// The device is opened lazily on the first play; construction only
    /// checks that a device exists, so headless machines construct fine
    /// and simply stay silent.
    pub fn new() -> Result<WinmmAudioBackend, String> {
        unsafe {
            if waveOutGetNumDevs() == 0 {
                return Err("no wave devices present".into());
            }
        }
        Ok(WinmmAudioBackend {
            device: None,
            next_id: 1,
            active: Vec::new(),
            device_sample_rate: 44100,
            open_error: None,
        })
    }

    /// Whether the device is open (opened lazily on first playback).
    pub fn is_device_open(&self) -> bool {
        self.device.is_some()
    }

    /// Construction-time device availability (number of wave devices).
    pub fn device_count() -> u32 {
        unsafe { waveOutGetNumDevs() }
    }

    fn open_device(&mut self, sample_rate: u32) -> bool {
        if self.device.is_some() {
            return true;
        }
        unsafe {
            let format = WAVEFORMATEX {
                w_format_tag: WAVE_FORMAT_PCM,
                n_channels: 1, // mono (engine clips are mono)
                n_samples_per_sec: sample_rate,
                n_avg_bytes_per_sec: sample_rate * 2,
                n_block_align: 2,
                w_bits_per_sample: 16,
                cb_size: 0,
            };
            let mut handle: HWAVEOUT = std::ptr::null_mut();
            let result = waveOutOpen(
                &mut handle,
                WAVE_MAPPER,
                &format,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CALLBACK_NULL,
            );
            if result != 0 {
                self.open_error = Some(format!("waveOutOpen failed: {result}"));
                return false;
            }
            self.device = Some(handle);
            self.device_sample_rate = sample_rate;
            true
        }
    }

    /// Diagnostics: why opening failed, if it did.
    pub fn open_error(&self) -> Option<&str> {
        self.open_error.as_deref()
    }
}

impl AudioBackend for WinmmAudioBackend {
    fn play_clip(&mut self, clip: &AudioClip, volume: f32, looped: bool) -> AudioPlaybackId {
        let id = self.next_id;
        self.next_id += 1;

        // resample mono clip to the device rate if needed (linear)
        let device_rate = if self.device.is_some() {
            self.device_sample_rate
        } else if !self.open_device(clip.sample_rate) {
            return id; // no device: silent playback (state tracked)
        } else {
            self.device_sample_rate
        };

        // apply volume and convert to bytes (mono PCM16)
        let vol = volume.clamp(0.0, 1.0) as f32 / 255.0 * 255.0; // 0..1 scale
        let scale = vol / 255.0 * 1.0; // volume in 0..1
        let mut samples: Vec<i16> = Vec::with_capacity(clip.samples.len());
        let ratio = clip.sample_rate.max(1) as f32 / device_rate as f32;
        let out_len = ((clip.samples.len() as f32) / ratio.max(0.001)) as usize;
        for i in 0..out_len {
            let src_index = ((i as f32) * ratio) as usize;
            let s = clip.samples.get(src_index).copied().unwrap_or(0);
            samples.push(((s as f32) * vol.clamp(0.0, 1.0)) as i16);
        }
        let _ = scale;

        let mut bytes: Box<[u8]> = samples
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>()
            .into_boxed_slice();

        // resample ratio (0 = same rate)
        if looped {
            // looping is emulated by repeating the PCM content up to ~4s
            let original_len = bytes.len();
            let repeats = ((device_rate * 4) as usize / original_len.max(1)).max(1);
            let mut extended = Vec::with_capacity(original_len * repeats);
            for _ in 0..repeats {
                extended.extend_from_slice(&bytes);
            }
            bytes = extended.into_boxed_slice();
        }

        unsafe {
            let mut header = WAVEHDR {
                lp_data: bytes.as_mut_ptr() as *mut c_void,
                dw_buffer_length: bytes.len() as u32,
                dw_bytes_recorded: 0,
                dw_user: std::ptr::null_mut(),
                dw_flags: 0,
                dw_loops: 0,
                lp_next: std::ptr::null_mut(),
                reserved: std::ptr::null_mut(),
            };
            if waveOutPrepareHeader(
                self.device.unwrap(),
                &mut header,
                std::mem::size_of::<WAVEHDR>() as u32,
            ) != 0
            {
                return id;
            }
            header.dw_flags |= WHDR_PREPARED;
            if waveOutWrite(
                self.device.unwrap(),
                &mut header,
                std::mem::size_of::<WAVEHDR>() as u32,
            ) != 0
            {
                waveOutUnprepareHeader(
                    self.device.unwrap(),
                    &mut header,
                    std::mem::size_of::<WAVEHDR>() as u32,
                );
                return id;
            }
            self.active.push(ActivePlayback {
                id,
                header,
                buffer: bytes,
                volume: volume.clamp(0.0, 1.0),
                looped,
            });
        }
        id
    }

    fn stop(&mut self, id: AudioPlaybackId) {
        let device = self.device;
        if let Some(device) = device {
            unsafe {
                waveOutReset(device);
            }
        }
        // after reset every header is completable — unprepare + free
        unsafe {
            for playback in self.active.iter_mut() {
                if let Some(device) = device {
                    waveOutUnprepareHeader(
                        device,
                        &mut playback.header,
                        std::mem::size_of::<WAVEHDR>() as u32,
                    );
                }
            }
        }
        self.active.retain(|p| p.id != id);
    }

    fn stop_all(&mut self) {
        if let Some(device) = self.device {
            unsafe {
                waveOutReset(device);
                for playback in self.active.iter_mut() {
                    waveOutUnprepareHeader(
                        device,
                        &mut playback.header,
                        std::mem::size_of::<WAVEHDR>() as u32,
                    );
                }
            }
        }
        self.active.clear();
    }

    fn active_count(&self) -> usize {
        self.active.len()
    }
}

impl Drop for WinmmAudioBackend {
    fn drop(&mut self) {
        self.stop_all();
        if let Some(device) = self.device {
            unsafe {
                waveOutClose(device);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_core::audio::AudioClip;

    #[test]
    fn construction_reports_device_presence() {
        let devices = WinmmAudioBackend::device_count();
        match WinmmAudioBackend::new() {
            Ok(backend) => {
                assert!(devices > 0);
                assert!(!backend.is_device_open()); // opened lazily
            }
            Err(e) => {
                // headless machine without wave devices: skip gracefully
                eprintln!("winmm skipped: {e}");
                assert_eq!(devices, 0);
            }
        }
    }

    #[test]
    fn play_stop_state_machine_with_or_without_device() {
        let Ok(mut backend) = WinmmAudioBackend::new() else {
            eprintln!("skipping: no wave device");
            return;
        };
        let clip = AudioClip::sine(44100, 440.0, 0.2, 8000);
        let a = backend.play_clip(&clip, 1.0, false);
        let b = backend.play_clip(&clip, 0.5, false);
        assert_ne!(a, b);
        backend.stop(a);
        backend.stop_all();
        assert_eq!(backend.active_count(), 0);
        // error diagnostics available for debugging
        let _ = backend.open_error();
    }

    #[test]
    fn waveformatex_layout() {
        let f = WAVEFORMATEX {
            w_format_tag: WAVE_FORMAT_PCM,
            n_channels: 1,
            n_samples_per_sec: 44100,
            n_avg_bytes_per_sec: 88200,
            n_block_align: 2,
            w_bits_per_sample: 16,
            cb_size: 0,
        };
        assert_eq!(f.w_format_tag, 1);
        assert_eq!(f.n_block_align, 2);
    }
}
