use crate::input::mouse::MouseAxis;
use crate::input::{GamePadAxis, GamePadButton, Key, MouseButton};

pub trait InputListener {
    fn key_changed(&mut self, key: Key, pressed: bool);

    fn mouse_button_changed(&mut self, button: MouseButton, pressed: bool);
    fn mouse_axis_changed(&mut self, axis: MouseAxis, x: f64, y: f64);

    fn game_pad_button_changed(&mut self, game_pad: usize, button: GamePadButton, pressed: bool);
    fn game_pad_axis_changed(&mut self, game_pad: usize, axis: GamePadAxis, x: f64, y: f64);
}
