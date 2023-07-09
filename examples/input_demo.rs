use spaghetti_engine::input::mouse::MouseAxis;
use spaghetti_engine::input::{
    GamePadAxis, GamePadButton, InputDispatcher, InputListener, Key, MouseButton,
};
use spaghetti_engine::settings::GameSettings;
use spaghetti_engine::spaghetti_entry_point;
use spaghetti_engine::window::{GameWindow, VsyncMode};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use glfw::{Context, fail_on_errors, WindowMode};

fn main() {
    spaghetti_entry_point!(input_demo());
}

struct TestListener;

impl InputListener for TestListener {
    fn key_changed(&mut self, key: Key, pressed: bool) {
        println!(
            "Key {} {}",
            key,
            if pressed { "pressed" } else { "released" }
        );
    }

    fn mouse_button_changed(&mut self, button: MouseButton, pressed: bool) {
        println!(
            "Mouse button {} {}",
            button,
            if pressed { "pressed" } else { "released" }
        );
    }

    fn mouse_position_changed(&mut self, x: f64, y: f64) {
        // println!("Mouse position changed to: (x: {}, y: {})", x, y);
    }

    fn mouse_scrolled(&mut self, x: f64, y: f64) {
        println!("Mouse scrolled: (x: {}, y: {})", x, y);
    }

    fn game_pad_button_changed(&mut self, game_pad: usize, button: GamePadButton, pressed: bool) {
        println!(
            "Game pad (index: {}) button {} {}",
            game_pad,
            button,
            if pressed { "pressed" } else { "released" }
        );
    }

    fn game_pad_axis_changed(&mut self, game_pad: usize, axis: GamePadAxis, x: f64, y: f64) {
        println!(
            "Game pad (index: {}) axis {} changed to: (x: {}, y: {})",
            game_pad, axis, x, y
        );
    }
}

fn input_demo() {
    let mut window = GameWindow::new(&GameSettings::new()).unwrap();
    window.make_context_current();
    window.set_visible(true);

    let dispatcher = Arc::new(InputDispatcher::new());
    let listener = TestListener {};
    dispatcher.register_listener(Box::new(listener));

    window.register_input_device(Arc::downgrade(&dispatcher));

    while !window.should_close() {
        thread::sleep(Duration::from_millis(1));
        dispatcher.update();
        window.swap();
    }
    window.no_context_current();
}
