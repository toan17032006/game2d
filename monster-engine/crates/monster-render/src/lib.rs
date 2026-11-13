//! Software 2D renderer for `monster-engine`.
//!
//! Implements [`monster_core::Renderer`] on top of a CPU framebuffer:
//! sprite blitting with alpha blending, tinting, flipping and sub-rect
//! source regions, solid rects, a camera transform, and clipping. The
//! finished frame is submitted to any [`monster_core::PresentTarget`] —
//! a Win32 window in production, or a headless capture in tests.
//!
//! Rendering is immediate (call order); layer/z sorting happens upstream
//! in the scene layer. A future GPU/3D backend implements the same
//! `Renderer` trait, so game code never changes.

pub mod framebuffer;
pub mod software;
pub mod text;

pub use framebuffer::Framebuffer;
pub use software::Software2DRenderer;
