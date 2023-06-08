use std::sync::Arc;
use spaghetti_engine::core::GameWindow;
use spaghetti_engine::settings::GameSettings;
use spaghetti_engine::utils::logger;

fn main() {
    let mut window = GameWindow::new(Arc::new(GameSettings::new()),
                                     &logger::GLOBAL_LOGGER).unwrap();
    window.do_loop();
}