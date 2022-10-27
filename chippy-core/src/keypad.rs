#[derive(Default)]
pub struct Keypad {
    pub keys: [bool; 16],
    pub last_pressed: u8,
}
