//! Typed event queue: the single channel through which platform,
//! input and gameplay communicate.

/// Events produced by the platform layer and consumed by engine/game code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineEvent {
    /// The user requested to close the window.
    WindowClosed,
    /// The window client area was resized.
    WindowResized {
        width: u32,
        height: u32,
    },
    /// The window lost focus.
    WindowFocusLost,
    WindowFocused,
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseMoved {
        x: f32,
        y: f32,
    },
    MouseButton {
        button: MouseButton,
        pressed: bool,
    },
    MouseWheel {
        delta: f32,
    },
}

use crate::input::{KeyCode, MouseButton};

/// FIFO queue of [`EngineEvent`]s, drained once per frame.
#[derive(Debug, Default)]
pub struct EventBus {
    queue: Vec<EngineEvent>,
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus { queue: Vec::new() }
    }

    /// Append an event to the end of the queue.
    pub fn push(&mut self, event: EngineEvent) {
        self.queue.push(event);
    }

    /// Remove and return all events in FIFO order.
    pub fn drain(&mut self) -> std::vec::IntoIter<EngineEvent> {
        std::mem::take(&mut self.queue).into_iter()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_order_and_drain() {
        let mut bus = EventBus::new();
        bus.push(EngineEvent::WindowClosed);
        bus.push(EngineEvent::KeyPressed(KeyCode::Escape));
        bus.push(EngineEvent::WindowFocused);
        assert_eq!(bus.len(), 3);
        let drained: Vec<_> = bus.drain().collect();
        assert_eq!(
            drained,
            vec![
                EngineEvent::WindowClosed,
                EngineEvent::KeyPressed(KeyCode::Escape),
                EngineEvent::WindowFocused,
            ]
        );
        assert!(bus.is_empty());
    }
}
