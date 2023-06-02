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
	Logger::debug("debug test");
	Logger::info("HELLO :D");
	Logger::warning("Hi hello haii heyy :3 heyy helloooo >_<");
	Logger::loading("loading something");
	Logger::error("really bad error");
	Logger::fatal("FATAL ERROR OMG EVERYTHING IS ON FIRE");

	let global = &logger::GLOBAL_LOGGER;

	let sub1 = Logger::from_str(global, "1");
	sub1.print_info("whar");

	let sub2 = Logger::from_str(&sub1, "2");
	sub2.print_error("xDDDD");

	let sub3 = Logger::from_str(&sub2, "3");
	sub3.print_error("AAAAAAAAAAAAAAAAA");

	let sub4 = Logger::from_str(&sub3, "4");
	sub4.print_error("AAAAAAAAAAAAAAAAA");

	let sub5 = Logger::from_str(&sub4, "5");
	sub5.print_error("AAAAAAAAAAAAAAAAA");

	let sub6 = Logger::from_str(&sub5, "6");
	sub6.print_error("AAAAAAAAAAAAAAAAA");

	sub5.print_error("A");
	sub1.print_error("A");
	global.print_error("A");
}