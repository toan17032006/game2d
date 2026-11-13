//! Asset loading for `monster-engine`.
//!
//! Everything is hand-rolled (zero external crates): CRC32/adler32
//! checksums, a full DEFLATE decompressor, and PNG/BMP image decoders that
//! feed the engine's RGBA8 [`Texture`]s. This is what makes the repo's
//! Tiny Swords PNG spritesheets usable.
//!
//! [`Assets`] is the registry: it decodes files once, caches them by path,
//! and hands out typed [`Handle`]s.

pub mod bmp;
pub mod crc;
pub mod inflate;
pub mod png;
pub mod registry;

#[cfg(test)]
pub mod fixtures;

pub use bmp::{decode_bmp, BmpError};
pub use png::{decode_png, PngError};
pub use registry::{AssetError, Assets, ImageData};
