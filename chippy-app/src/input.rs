use chippy_core::Keypad;
use miniquad::KeyCode;

pub(crate) trait InputHandler {
    /// Returns whether the key event was caught or not.
    fn key_event(&mut self, event: KeyEvent, keycode: KeyCode) -> bool;
}

impl InputHandler for Keypad {
    /// Returns whether the key event was caught or not.
    fn key_event(&mut self, event: KeyEvent, keycode: KeyCode) -> bool {
        let key = match keycode {
            KeyCode::Key1 => 0x1,
            KeyCode::Key2 => 0x2,
            KeyCode::Key3 => 0x3,
            KeyCode::Key4 => 0xC,
            KeyCode::Q => 0x4,
            KeyCode::W => 0x5,
            KeyCode::E => 0x6,
            KeyCode::R => 0xD,
            KeyCode::A => 0x7,
            KeyCode::S => 0x8,
            KeyCode::D => 0x9,
            KeyCode::F => 0xE,
            KeyCode::Z => 0xA,
            KeyCode::X => 0x0,
            KeyCode::C => 0xB,
            KeyCode::V => 0xF,
            _ => return false,
        };

        self.keys[key] = bool::from(event);
        self.last_pressed = key as u8;

        true
    }
}

#[derive(Clone)]
pub enum KeyEvent {
    KeyUp,
    KeyDown,
}

impl From<KeyEvent> for bool {
    fn from(f: KeyEvent) -> bool {
        match f {
            KeyEvent::KeyUp => false,
            KeyEvent::KeyDown => true,
        }
    }
}
