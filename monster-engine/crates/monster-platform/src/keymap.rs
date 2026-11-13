//! Virtual-key code → engine [`KeyCode`] mapping (pure, unit-tested).

use monster_core::input::KeyCode;

/// Translate a Win32 virtual-key code into an engine key code.
///
/// Modifiers collapse to their left variants (Win32 exposes generic
/// VK_SHIFT/VK_CONTROL; distinguishing sides requires scan codes, which the
/// foundation does not need).
pub fn vk_to_keycode(vk: u32) -> KeyCode {
    match vk {
        0x41..=0x5A => letter((vk - 0x41) as u8),
        0x30..=0x39 => digit((vk - 0x30) as u8),
        crate::ffi::VK_ESCAPE => KeyCode::Escape,
        crate::ffi::VK_SPACE => KeyCode::Space,
        crate::ffi::VK_RETURN => KeyCode::Enter,
        crate::ffi::VK_TAB => KeyCode::Tab,
        crate::ffi::VK_BACK => KeyCode::Backspace,
        crate::ffi::VK_LEFT => KeyCode::Left,
        crate::ffi::VK_RIGHT => KeyCode::Right,
        crate::ffi::VK_UP => KeyCode::Up,
        crate::ffi::VK_DOWN => KeyCode::Down,
        crate::ffi::VK_SHIFT => KeyCode::LeftShift,
        crate::ffi::VK_CONTROL => KeyCode::LeftCtrl,
        _ => KeyCode::Unknown,
    }
}

fn letter(index: u8) -> KeyCode {
    match index {
        0 => KeyCode::A,
        1 => KeyCode::B,
        2 => KeyCode::C,
        3 => KeyCode::D,
        4 => KeyCode::E,
        5 => KeyCode::F,
        6 => KeyCode::G,
        7 => KeyCode::H,
        8 => KeyCode::I,
        9 => KeyCode::J,
        10 => KeyCode::K,
        11 => KeyCode::L,
        12 => KeyCode::M,
        13 => KeyCode::N,
        14 => KeyCode::O,
        15 => KeyCode::P,
        16 => KeyCode::Q,
        17 => KeyCode::R,
        18 => KeyCode::S,
        19 => KeyCode::T,
        20 => KeyCode::U,
        21 => KeyCode::V,
        22 => KeyCode::W,
        23 => KeyCode::X,
        24 => KeyCode::Y,
        _ => KeyCode::Z,
    }
}

fn digit(index: u8) -> KeyCode {
    match index {
        0 => KeyCode::Num0,
        1 => KeyCode::Num1,
        2 => KeyCode::Num2,
        3 => KeyCode::Num3,
        4 => KeyCode::Num4,
        5 => KeyCode::Num5,
        6 => KeyCode::Num6,
        7 => KeyCode::Num7,
        8 => KeyCode::Num8,
        _ => KeyCode::Num9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_digits_map() {
        assert_eq!(vk_to_keycode(0x41), KeyCode::A);
        assert_eq!(vk_to_keycode(0x5A), KeyCode::Z);
        assert_eq!(vk_to_keycode(0x30), KeyCode::Num0);
        assert_eq!(vk_to_keycode(0x39), KeyCode::Num9);
    }

    #[test]
    fn control_keys_map() {
        assert_eq!(vk_to_keycode(0x1B), KeyCode::Escape);
        assert_eq!(vk_to_keycode(0x20), KeyCode::Space);
        assert_eq!(vk_to_keycode(0x0D), KeyCode::Enter);
        assert_eq!(vk_to_keycode(0x25), KeyCode::Left);
        assert_eq!(vk_to_keycode(0x26), KeyCode::Up);
        assert_eq!(vk_to_keycode(0x27), KeyCode::Right);
        assert_eq!(vk_to_keycode(0x28), KeyCode::Down);
        assert_eq!(vk_to_keycode(0x10), KeyCode::LeftShift);
        assert_eq!(vk_to_keycode(0x11), KeyCode::LeftCtrl);
    }

    #[test]
    fn unknown_falls_back() {
        assert_eq!(vk_to_keycode(0xFF), KeyCode::Unknown);
        assert_eq!(vk_to_keycode(0x13), KeyCode::Unknown); // VK_PAUSE
    }
}
