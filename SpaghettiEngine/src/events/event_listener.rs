use std::{sync};
use crate::core::Game;
use crate::events::{GameEvent};
use crate::utils::types::ObjectId;

pub trait EventListener {
	fn handle_event(&mut self, game: sync::Weak<Game>, event: &mut Box<dyn GameEvent>);
	fn get_target_id(&self) -> ObjectId {
		0
	}
}