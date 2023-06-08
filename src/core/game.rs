use std::{sync, thread};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::ThreadId;
use once_cell::sync::Lazy;
use crate::events::event_dispatcher::EventDispatcher;
use crate::settings::GameSettings;
use crate::utils::Logger;
use crate::world::client_state::ClientState;
use crate::world::GameState;

static GAMES: RwLock<Vec<sync::Weak<Game>>> = RwLock::new(Vec::new());
static LINKS: Lazy<RwLock<HashMap<ThreadId, sync::Weak<Game>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Game {
	event_dispatcher: EventDispatcher,
	game_state: GameState,
	client_state: ClientState,
	game_settings: Arc<GameSettings>,
	logger: Arc<Logger>,
	is_client: bool
}

impl Game {

	pub fn get_instance() -> sync::Weak<Game> {
		let links = LINKS.read().unwrap();
		match links.get(&thread::current().id()) {
			Some(game) => game.clone(),
			None => sync::Weak::new()
		}
	}

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

	pub fn get_logger(&self) -> Arc<Logger> {
		self.logger.clone()
	}

	pub fn get_index(&self) -> u64 {
		0
	}
}