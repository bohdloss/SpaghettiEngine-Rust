use crate::input::mouse::MouseAxis;
use crate::input::{GamePadAxis, GamePadButton, Key, MouseButton};
use array_init::array_init;
use std::cell::UnsafeCell;

pub const NUM_GAME_PADS: usize = 16;

#[derive(Copy, Clone)]
enum MouseEventType {
    ButtonChange,
    Move,
    Scroll,
}

// TODO BUY A PS4 / XBOX CONTROLLER AND TEST INPUTS

pub struct KeyboardState {
    pub keys: [bool; Key::size()],
}

impl KeyboardState {
    fn new() -> Self {
        Self {
            keys: [false; Key::size()],
        }
    }
}

pub struct MouseState {
    pub buttons: [bool; MouseButton::size()],
    pub axis: [f64; MouseAxis::size()],
}

impl MouseState {
    fn new() -> Self {
        Self {
            buttons: [false; MouseButton::size()],
            axis: [0.0; MouseAxis::size()],
        }
    }
}

pub struct GamePadState {
    pub buttons: [bool; GamePadButton::size()],
    pub axis: [f64; GamePadAxis::size()],
}

impl GamePadState {
    fn new() -> Self {
        Self {
            buttons: [false; GamePadButton::size()],
            axis: [0.0; GamePadAxis::size()],
        }
    }
}

struct InputBuffers {
    keyboard: KeyboardState,
    mouse: MouseState,
    game_pad: [GamePadState; NUM_GAME_PADS],
}

impl InputBuffers {
    fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            game_pad: array_init(|_| GamePadState::new()),
        }
    }
}

pub struct InputDispatcher {
    old: UnsafeCell<InputBuffers>,
    new: UnsafeCell<InputBuffers>,
}

impl InputDispatcher {
    pub fn new() -> Self {
        Self {
            old: UnsafeCell::new(InputBuffers::new()),
            new: UnsafeCell::new(InputBuffers::new()),
        }
    }

    pub fn update(&self) {
        for (i, &new_val) in self.keyboard_state().keys.iter().enumerate() {
            // Update cache
            let old_state = self.old_keyboard_state();
            if new_val != old_state.keys[i] {
                old_state.keys[i] = new_val;
            }

            // TODO Dispatch input
        }

        for (i, &new_val) in self.mouse_state().buttons.iter().enumerate() {
            // Update cache
            let old_state = self.old_mouse_state();
            if new_val != old_state.buttons[i] {
                old_state.buttons[i] = new_val;
            }

            // TODO ...
        }

        for (i, &new_val) in self.mouse_state().axis.iter().enumerate() {
            // Update cache
            let old_state = self.old_mouse_state();
            if new_val != old_state.axis[i] {
                old_state.axis[i] = new_val;
            }

            // TODO ...
        }

        for game_pad_index in 0..NUM_GAME_PADS {
            for (i, &new_val) in self
                .game_pad_state(game_pad_index)
                .buttons
                .iter()
                .enumerate()
            {
                // Update cache
                let old_state = self.old_game_pad_state(game_pad_index);
                if new_val != old_state.buttons[i] {
                    old_state.buttons[i] = new_val;
                }

                // TODO ...
            }

            for (i, &new_val) in self.game_pad_state(game_pad_index).axis.iter().enumerate() {
                // Update cache
                let old_state = self.old_game_pad_state(game_pad_index);
                if new_val != old_state.axis[i] {
                    old_state.axis[i] = new_val;
                }

                // TODO ...
            }
        }
    }

    pub fn keyboard_state(&self) -> &mut KeyboardState {
        unsafe { &mut (*self.new.get()).keyboard }
    }

    fn old_keyboard_state(&self) -> &mut KeyboardState {
        unsafe { &mut (*self.old.get()).keyboard }
    }

    pub fn mouse_state(&self) -> &mut MouseState {
        unsafe { &mut (*self.new.get()).mouse }
    }

    fn old_mouse_state(&self) -> &mut MouseState {
        unsafe { &mut (*self.old.get()).mouse }
    }

    pub fn game_pad_state(&self, index: usize) -> &mut GamePadState {
        unsafe { &mut (*self.new.get()).game_pad[index] }
    }

    fn old_game_pad_state(&self, index: usize) -> &mut GamePadState {
        unsafe { &mut (*self.old.get()).game_pad[index] }
    }
}
