use crate::input::mouse::MouseAxis;
use crate::input::{GamePadAxis, GamePadButton, InputListener, Key, MouseButton};
use crate::utils::id_type::id_type;
use array_init::array_init;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::Mutex;

pub const NUM_GAME_PADS: usize = 16;

// TODO BUY A PS4 / XBOX CONTROLLER AND TEST INPUTS

id_type!(ListenerHandle);

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
    pub axis: [(f64, f64); MouseAxis::size()],
}

impl MouseState {
    fn new() -> Self {
        Self {
            buttons: [false; MouseButton::size()],
            axis: [(0.0, 0.0); MouseAxis::size()],
        }
    }
}

pub struct GamePadState {
    pub buttons: [bool; GamePadButton::size()],
    pub axis: [(f64, f64); GamePadAxis::size()],
}

impl GamePadState {
    fn new() -> Self {
        Self {
            buttons: [false; GamePadButton::size()],
            axis: [(0.0, 0.0); GamePadAxis::size()],
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
    listeners: Mutex<HashMap<ListenerHandle, Box<dyn InputListener>>>,
}

impl InputDispatcher {
    pub fn new() -> Self {
        Self {
            old: UnsafeCell::new(InputBuffers::new()),
            new: UnsafeCell::new(InputBuffers::new()),
            listeners: Mutex::new(HashMap::new()),
        }
    }

    pub fn update(&self) {
        // Key update
        for (i, &new_val) in self.keyboard_state().keys.iter().enumerate() {
            let new_val = new_val;
            let old_state = self.old_keyboard_state();
            if new_val != old_state.keys[i] {
                // Update cache
                old_state.keys[i] = new_val;

                // Fire events
                self.with_all_listeners(|listener| {
                    listener.key_changed(Key::from_usize(i), new_val)
                });
            }
        }

        // Mouse buttons update
        for (i, &new_val) in self.mouse_state().buttons.iter().enumerate() {
            let new_val = new_val;
            let old_state = self.old_mouse_state();
            if new_val != old_state.buttons[i] {
                old_state.buttons[i] = new_val;

                self.with_all_listeners(|listener| {
                    listener.mouse_button_changed(MouseButton::from_usize(i), new_val)
                });
            }
        }

        // Mouse position / wheel update
        let new_state = &mut self.mouse_state().axis;
        let old_state = &mut self.old_mouse_state().axis;

        let position_index = MouseAxis::Position.index();
        let wheel_index = MouseAxis::Wheel.index();

        if new_state[position_index] != old_state[position_index] {
            old_state[position_index] = new_state[position_index];

            self.with_all_listeners(|listener| {
                listener.mouse_axis_changed(
                    MouseAxis::Position,
                    new_state[position_index].0,
                    new_state[position_index].1,
                )
            });
        }

        // TODO Better scroll handling

        let new_scroll_val = new_state[wheel_index];
        if new_scroll_val != old_state[wheel_index] {
            old_state[wheel_index] = new_scroll_val;

            self.with_all_listeners(|listener| {
                listener.mouse_axis_changed(MouseAxis::Wheel, new_scroll_val.0, new_scroll_val.1)
            });
        }
        new_state[wheel_index] = (0.0, 0.0);

        // Must iterate over all game pads and...
        for game_pad_index in 0..NUM_GAME_PADS {
            // ...fire button events
            for (i, &new_val) in self
                .game_pad_state(game_pad_index)
                .buttons
                .iter()
                .enumerate()
            {
                let new_val = new_val;
                let old_state = self.old_game_pad_state(game_pad_index);
                if new_val != old_state.buttons[i] {
                    old_state.buttons[i] = new_val;

                    self.with_all_listeners(|listener| {
                        listener.game_pad_button_changed(
                            game_pad_index,
                            GamePadButton::from_usize(i),
                            new_val,
                        )
                    });
                }
            }

            // ...fire axis events
            for (i, &new_val) in self.game_pad_state(game_pad_index).axis.iter().enumerate() {
                let new_val = new_val;
                let old_state = self.old_game_pad_state(game_pad_index);
                if new_val != old_state.axis[i] {
                    old_state.axis[i] = new_val;

                    self.with_all_listeners(|listener| {
                        listener.game_pad_axis_changed(
                            game_pad_index,
                            GamePadAxis::from_usize(i),
                            new_val.0,
                            new_val.1,
                        )
                    });
                }
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

    fn with_all_listeners<T>(&self, f: T)
    where
        T: Fn(&mut dyn InputListener),
    {
        for (_, listener) in self.listeners.lock().unwrap().iter_mut() {
            f(listener.as_mut());
        }
    }

    pub fn register_listener(&self, listener: Box<dyn InputListener>) -> ListenerHandle {
        let mut list = self.listeners.lock().unwrap();
        let id = ListenerHandle::new();
        list.insert(id, listener);
        id
    }

    pub fn unregister_listener(&self, handle: ListenerHandle) {
        self.listeners.lock().unwrap().remove(&handle);
    }
}
