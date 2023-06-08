use std::sync;
use std::sync::Arc;
use crate::core::GameWindow;
use crate::settings::GameSettings;
use crate::utils::{Logger, logger};

pub mod core;
pub mod demo;
pub mod utils;
pub mod settings;
pub mod events;
pub mod networking;
pub mod world;
pub mod input;

pub fn main() {
    let mut window = GameWindow::new(Arc::new(GameSettings::new()),
                                     &logger::GLOBAL_LOGGER).unwrap();
    window.do_loop();
}