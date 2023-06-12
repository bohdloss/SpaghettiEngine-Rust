use crate::core::Game;
use crate::events::GameEvent;
use std::sync;

pub trait EventListener: Send {
    fn handle_event(&self, game: sync::Weak<Game>, event: &mut Box<dyn GameEvent>);
}
