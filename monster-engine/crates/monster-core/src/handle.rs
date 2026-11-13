//! Typed handles: safe references into engine-managed resource pools.

use std::fmt;
use std::marker::PhantomData;

/// A typed index into a resource pool (e.g. [`crate::graphics::Texture`]).
///
/// Handles are `Copy`, cheap, and type-safe: a `Handle<Texture>` cannot be
/// confused with a `Handle<AudioClip>`. The engine owns the actual data;
/// game code passes handles around.
///
/// `Copy`/`Clone` are implemented manually (not derived) so they hold for
/// every `T` — a handle copies the index, never the resource.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Handle<T> {
    /// Create a handle from a raw index (used by resource owners only).
    pub fn from_index(index: u32) -> Handle<T> {
        Handle {
            index,
            _marker: PhantomData,
        }
    }

    /// Raw index (used by resource owners only).
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl<T> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle({})", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::Texture;

    #[test]
    fn handles_are_typed_and_copy() {
        let h = Handle::<Texture>::from_index(7);
        assert_eq!(h.index(), 7);
        let h2 = h; // Copy
        assert_eq!(h, h2);
        // different T => different type, enforced at compile time
        let _other = Handle::<crate::audio::AudioClip>::from_index(7);
        assert_eq!(format!("{h}"), "Handle(7)");
    }
}
