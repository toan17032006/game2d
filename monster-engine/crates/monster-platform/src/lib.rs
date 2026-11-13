//! Platform layer for `monster-engine`: the only place that talks to the
//! operating system.
//!
//! Win32 FFI declarations live in [`ffi`] (hand-written, zero external
//! crates). [`window::Window`] is the production window (Win32 + GDI
//! present), [`window::NullWindow`] is the headless substitute used by
//! tests and `--headless` runs. Both implement [`window::PlatformWindow`],
//! which bundles the engine-facing surface (open/poll/size/close) with the
//! renderer-facing [`PresentTarget`] so the engine holds a single
//! `Box<dyn PlatformWindow>` per run.
//!
//! Porting the engine to another OS means adding a sibling implementation
//! of `PlatformWindow` here — nothing above this crate changes.

pub mod ffi;
pub mod gl;
pub mod keymap;
pub mod window;

pub use gl::GlContext;
pub use keymap::vk_to_keycode;
pub use window::{NullWindow, PlatformError, PlatformWindow, Window, WindowOptions};
