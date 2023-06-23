use crate::input::{GamePad, GamePadAxis, Keyboard, KeyboardMods, Mouse};

#[derive(Copy, Clone)]
enum MouseEventType {
    ButtonChange,
    Move,
    Scroll,
}

// TODO BUY A PS4 / XBOX CONTROLLER AND TEST INPUTS

pub struct InputDispatcher {
    pub keyboard_state: [bool; Keyboard::size()],
    pub keyboard_mods_state: [bool; KeyboardMods::size()],
    pub mouse_state: [bool; Mouse::size()],
    pub game_pad_state: [bool; GamePad::size()],
    pub game_pad_axis_state: [f32; GamePadAxis::size()],
}

impl InputDispatcher {}
