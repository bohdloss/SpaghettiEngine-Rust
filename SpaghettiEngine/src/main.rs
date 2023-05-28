use std::thread;
use std::time::{Duration, SystemTime};
use chrono::{Datelike, Timelike, Utc};
use crate::utils::Logger;
use crate::utils::logger::Severity;

pub mod core;
pub mod demo;
pub mod utils;
pub mod settings;
pub mod events;
pub mod networking;
pub mod world;
pub mod input;

pub fn main() {
	Logger::debug("debug test");
	Logger::info("HELLO :D");
	Logger::warning("Hi hello haii heyy :3 heyy helloooo >_<");
	Logger::loading("loading something");
	Logger::error("really bad error");
	Logger::fatal("FATAL ERROR OMG EVERYTHING IS ON FIRE");
}