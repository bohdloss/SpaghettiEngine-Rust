use std::{sync};
use crate::core::Game;
use crate::events::{GameEvent};

pub trait EventListener {
	fn handle_event(&self, game: sync::Weak<Game>, event: &mut Box<dyn GameEvent>);
}