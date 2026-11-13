//! Core contracts and services for `monster-engine`.
//!
//! This crate owns the engine's *interfaces*: the [`Renderer`], [`AudioBackend`]
//! and [`PresentTarget`] traits live here so concrete backends (software 2D
//! now, GPU/3D later) can be added as new crates without touching the core.
//! It also provides the shared services every subsystem uses: the event bus,
//! the fixed-timestep clock, input state and leveled logging.

pub mod audio;
pub mod events;
pub mod graphics;
pub mod handle;
pub mod input;
pub mod log;
pub mod mesh;
pub mod time;

pub use audio::{AudioBackend, AudioClip, AudioPlaybackId};
pub use events::{EngineEvent, EventBus};
pub use graphics::{Camera2D, Camera3D, DrawOptions, PresentTarget, Renderer, Renderer3D, Texture};
pub use handle::Handle;
pub use input::{InputState, KeyCode, MouseButton};
pub use mesh::Mesh3D;
pub use time::{FixedClock, Stopwatch};
