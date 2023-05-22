use crate::dispatcher::FunctionDispatcher;
use crate::events::event_dispatcher::EventDispatcher;
use crate::world::GameState;

pub struct Game {
	event_dispatcher: EventDispatcher,
	game_state: GameState,
	primary_dispatcher: FunctionDispatcher,
	is_client: bool
}

impl Game {

	pub fn get_event_dispatcher(&self) -> &EventDispatcher {
		&self.event_dispatcher
	}

	pub fn get_primary_dispatcher(&self) -> &FunctionDispatcher {
		&self.primary_dispatcher
	}

	pub fn is_client(&self) -> bool {
		self.is_client
	}

	pub fn get_game_state(&self) -> &GameState {
		&self.game_state
	}

}