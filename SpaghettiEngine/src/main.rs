use std::sync;
use crate::utils::Logger;

pub mod core;
pub mod demo;
pub mod utils;
pub mod settings;
pub mod events;
pub mod networking;
pub mod world;
pub mod input;

pub fn main() {
	let logger = Logger::new(sync::Weak::new());
	logger.info("Hello");
}