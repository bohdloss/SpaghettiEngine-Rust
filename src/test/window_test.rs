use std::sync::Arc;
use crate::core::*;
use crate::settings::GameSettings;

#[test]
fn window() {
    // Setup settings
    let default_settings = Arc::new(GameSettings::new());

    // Initialize window
    let mut settings = default_settings.clone();
    let mut window = init_window(&settings);

    // Do the testing testful tests
    window.set_visible(true);
    while !window.should_close() {
        window.poll_events();
    }
}

fn init_window(settings: &Arc<GameSettings>) -> GameWindow {
    match GameWindow::new(settings) {
        Ok(window) => {
            window
        },
        Err(error) => {
            panic!("{}", error);
        }
    }
}