use crate::input::InputDispatcher;

pub trait InputDevice {
    fn update_input_state(dispatcher: &InputDispatcher);
}