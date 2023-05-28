use crate::events::event_dispatcher::EventDispatcher;
use crate::settings::GameSettings;
use crate::world::GameState;

pub struct Game {
	event_dispatcher: EventDispatcher,
	game_state: GameState,
	game_settings: GameSettings,
	is_client: bool
}

impl Game {

	pub fn get_event_dispatcher(&self) -> &EventDispatcher {
		&self.event_dispatcher
	}

	pub fn is_client(&self) -> bool {
		self.is_client
	}

	pub fn get_game_state(&self) -> &GameState {
		&self.game_state
	}

	pub fn get_settings(&self) -> &GameSettings {
		&self.game_settings
	}

	pub fn get_index(&self) -> u64 {
		0
	}
}