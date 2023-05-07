use crate::core::Game;
use crate::events::GameEvent;
use crate::utils::types::*;

pub trait EventListener {
	fn handle_event(&mut self, game: ArcRwLock<Game>, event: Box<dyn GameEvent>);
}