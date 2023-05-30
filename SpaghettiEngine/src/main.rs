use std::ops::Sub;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use chrono::{Datelike, Timelike, Utc};
use crate::utils::{Logger, logger};
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

	let global = &logger::GLOBAL_LOGGER;

	let sub1 = Logger::push("1");
	sub1.print_info("whar");

	let sub2 = Logger::push("2");
	sub2.print_error("xDDDD");

	let sub3 = Logger::push("3");
	sub3.print_error("AAAAAAAAAAAAAAAAA");

	let sub4 = Logger::push("4");
	sub4.print_error("AAAAAAAAAAAAAAAAA");

	let sub5 = Logger::push("5");
	sub5.print_error("AAAAAAAAAAAAAAAAA");

	let sub6 = Logger::push("6");
	sub6.print_error("AAAAAAAAAAAAAAAAA");
	sub6.print_info("before drop");
	drop(sub2);
	sub6.print_info("dropped sub2");

	sub5.print_error("A");
	sub1.print_error("A");
	global.print_error("A");
}