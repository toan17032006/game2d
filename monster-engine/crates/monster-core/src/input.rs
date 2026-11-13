//! Keyboard and mouse: key codes and a frame-local input state snapshot.

use crate::events::EngineEvent;
use crate::events::EventBus;

/// Keyboard keys the engine recognizes (foundation subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Unknown,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
}

impl KeyCode {
    /// Dense index used to store per-key flags.
    pub fn index(self) -> usize {
        match self {
            KeyCode::Unknown => 0,
            KeyCode::A => 1,
            KeyCode::B => 2,
            KeyCode::C => 3,
            KeyCode::D => 4,
            KeyCode::E => 5,
            KeyCode::F => 6,
            KeyCode::G => 7,
            KeyCode::H => 8,
            KeyCode::I => 9,
            KeyCode::J => 10,
            KeyCode::K => 11,
            KeyCode::L => 12,
            KeyCode::M => 13,
            KeyCode::N => 14,
            KeyCode::O => 15,
            KeyCode::P => 16,
            KeyCode::Q => 17,
            KeyCode::R => 18,
            KeyCode::S => 19,
            KeyCode::T => 20,
            KeyCode::U => 21,
            KeyCode::V => 22,
            KeyCode::W => 23,
            KeyCode::X => 24,
            KeyCode::Y => 25,
            KeyCode::Z => 26,
            KeyCode::Num0 => 27,
            KeyCode::Num1 => 28,
            KeyCode::Num2 => 29,
            KeyCode::Num3 => 30,
            KeyCode::Num4 => 31,
            KeyCode::Num5 => 32,
            KeyCode::Num6 => 33,
            KeyCode::Num7 => 34,
            KeyCode::Num8 => 35,
            KeyCode::Num9 => 36,
            KeyCode::Space => 37,
            KeyCode::Enter => 38,
            KeyCode::Escape => 39,
            KeyCode::Tab => 40,
            KeyCode::Backspace => 41,
            KeyCode::Left => 42,
            KeyCode::Right => 43,
            KeyCode::Up => 44,
            KeyCode::Down => 45,
            KeyCode::LeftShift => 46,
            KeyCode::RightShift => 47,
            KeyCode::LeftCtrl => 48,
            KeyCode::RightCtrl => 49,
        }
    }

    /// All keys, useful for iteration in tests/tools.
    pub fn all() -> impl Iterator<Item = KeyCode> {
        [
            KeyCode::Unknown,
            KeyCode::A,
            KeyCode::B,
            KeyCode::C,
            KeyCode::D,
            KeyCode::E,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::I,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::M,
            KeyCode::N,
            KeyCode::O,
            KeyCode::P,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::S,
            KeyCode::T,
            KeyCode::U,
            KeyCode::V,
            KeyCode::W,
            KeyCode::X,
            KeyCode::Y,
            KeyCode::Z,
            KeyCode::Num0,
            KeyCode::Num1,
            KeyCode::Num2,
            KeyCode::Num3,
            KeyCode::Num4,
            KeyCode::Num5,
            KeyCode::Num6,
            KeyCode::Num7,
            KeyCode::Num8,
            KeyCode::Num9,
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::Backspace,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::LeftShift,
            KeyCode::RightShift,
            KeyCode::LeftCtrl,
            KeyCode::RightCtrl,
        ]
        .into_iter()
    }
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    /// Dense index offset after the last keyboard key.
    const BASE: usize = 50;

    fn index(self) -> usize {
        match self {
            MouseButton::Left => Self::BASE,
            MouseButton::Right => Self::BASE + 1,
            MouseButton::Middle => Self::BASE + 2,
        }
    }
}

const TOTAL_INPUT_SLOTS: usize = 53;

/// Snapshot of keyboard/mouse state for the current frame.
///
/// Feed it the frame's events via [`InputState::begin_frame`]; afterwards
/// game code queries the `is_down` / `was_pressed` / `was_released` families.
#[derive(Debug)]
pub struct InputState {
    held: Vec<bool>,
    pressed: Vec<bool>,
    released: Vec<bool>,
    mouse_x: f32,
    mouse_y: f32,
    wheel_delta: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            held: vec![false; TOTAL_INPUT_SLOTS],
            pressed: vec![false; TOTAL_INPUT_SLOTS],
            released: vec![false; TOTAL_INPUT_SLOTS],
            mouse_x: 0.0,
            mouse_y: 0.0,
            wheel_delta: 0.0,
        }
    }

    /// Clear per-frame flags, then fold the event queue into the new state.
    pub fn begin_frame(&mut self, events: &mut EventBus) {
        for f in self.pressed.iter_mut() {
            *f = false;
        }
        for f in self.released.iter_mut() {
            *f = false;
        }
        self.wheel_delta = 0.0;
        for event in events.drain() {
            match event {
                EngineEvent::KeyPressed(k) => {
                    let i = k.index();
                    self.held[i] = true;
                    self.pressed[i] = true;
                }
                EngineEvent::KeyReleased(k) => {
                    let i = k.index();
                    self.held[i] = false;
                    self.released[i] = true;
                }
                EngineEvent::MouseMoved { x, y } => {
                    self.mouse_x = x;
                    self.mouse_y = y;
                }
                EngineEvent::MouseButton { button, pressed } => {
                    let i = button.index();
                    self.held[i] = pressed;
                    self.pressed[i] = pressed;
                    self.released[i] = !pressed;
                }
                EngineEvent::MouseWheel { delta } => self.wheel_delta += delta,
                _ => {}
            }
        }
    }

    /// True while the key is physically held down.
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.held[key.index()]
    }

    /// True only on the frame the key went down.
    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.pressed[key.index()]
    }

    /// True only on the frame the key came up.
    pub fn was_released(&self, key: KeyCode) -> bool {
        self.released[key.index()]
    }

    /// True while the mouse button is held down.
    pub fn mouse_down(&self, button: MouseButton) -> bool {
        self.held[button.index()]
    }

    /// True only on the frame the mouse button went down.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.pressed[button.index()]
    }

    /// True only on the frame the mouse button came up.
    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.released[button.index()]
    }

    /// Last known mouse position in window coordinates.
    pub fn mouse_position(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y)
    }

    /// Accumulated wheel delta for this frame.
    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_hold_release_cycle() {
        let mut input = InputState::new();
        let mut bus = EventBus::new();
        bus.push(EngineEvent::KeyPressed(KeyCode::Space));
        input.begin_frame(&mut bus);
        assert!(input.was_pressed(KeyCode::Space));
        assert!(input.is_down(KeyCode::Space));
        // second frame: still held, no longer pressed
        input.begin_frame(&mut bus);
        assert!(!input.was_pressed(KeyCode::Space));
        assert!(input.is_down(KeyCode::Space));
        // release
        bus.push(EngineEvent::KeyReleased(KeyCode::Space));
        input.begin_frame(&mut bus);
        assert!(input.was_released(KeyCode::Space));
        assert!(!input.is_down(KeyCode::Space));
        input.begin_frame(&mut bus);
        assert!(!input.was_released(KeyCode::Space));
        assert!(!input.is_down(KeyCode::Space));
    }

    #[test]
    fn mouse_buttons_and_motion() {
        let mut input = InputState::new();
        let mut bus = EventBus::new();
        bus.push(EngineEvent::MouseMoved { x: 42.0, y: 7.0 });
        bus.push(EngineEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
        });
        bus.push(EngineEvent::MouseWheel { delta: 1.0 });
        bus.push(EngineEvent::MouseWheel { delta: -0.5 });
        input.begin_frame(&mut bus);
        let (x, y) = input.mouse_position();
        assert_eq!(x, 42.0);
        assert_eq!(y, 7.0);
        assert_eq!(input.wheel_delta(), 0.5);
        assert!(input.mouse_down(MouseButton::Left));
        assert!(input.mouse_pressed(MouseButton::Left));
        assert!(!input.mouse_released(MouseButton::Left));
        bus.push(EngineEvent::MouseButton {
            button: MouseButton::Left,
            pressed: false,
        });
        input.begin_frame(&mut bus);
        assert!(input.mouse_released(MouseButton::Left));
        assert!(!input.mouse_down(MouseButton::Left));
    }

    #[test]
    fn dense_indices_unique() {
        let all: Vec<_> = KeyCode::all().collect();
        for (i, k) in all.iter().enumerate() {
            assert_eq!(k.index(), i, "index mismatch for {:?}", k);
        }
        for (offset, b) in [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
            .iter()
            .enumerate()
        {
            assert_eq!(b.index(), all.len() + offset);
        }
    }
}
