pub trait BeginEndPlay {

	fn on_begin_play(&mut self) -> Result<(), BeginError>;
	fn on_end_play(&mut self);

}

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct BeginError {
	message: String
}

impl Debug for BeginError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self::Display::fmt(self, f)
	}
}

impl Display for BeginError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Error within Game Mode: {}", self.message)
	}
}

impl Error for BeginError {
}

impl BeginError {

	pub fn new(message: &str) -> Self {
		Self {
			message: message.to_string()
		}
	}

}