use std::collections::HashMap;
use std::sync;
use crate::core::Game;
use crate::input::controller::Controller;
use crate::networking::token::Token;
use crate::utils::types::*;
use crate::world::empty_game_mode::EmptyGameMode;
use crate::world::game_mode::GameMode;
use crate::world::level::Level;
use crate::world::{BeginEndPlay, Update};

pub struct GameState {
	game: sync::Weak<Game>,
	game_mode: Box<dyn GameMode>,
	game_mode_initialized: bool,
	levels: HashMap<String, Level>,
	players: HashMap<Token, Controller>,
	tick_multiplier: float,
	needs_replication: bool
}

impl Update for GameState {
	fn update(&mut self, delta: float) {
		// Check if the game mode needs initialization
		if !self.game_mode_initialized {
			match self.game_mode.on_begin_play() {
				Ok(_) => {
					self.game_mode_initialized = true;
				},
				Err(error) => {
					println!("Error while initializing Game Mode, using fallback dummy game mode. Error: {}", error);
					self.set_game_mode(Box::new(EmptyGameMode::new()));
				}
			}
		}

		self.game_mode.update(delta);

		for level in self.levels.iter_mut() {
			if level.1.is_active() {
				level.1.update(delta);
			}
		}
	}
}

impl GameState {

	pub fn new(game: sync::Weak<Game>) -> Self {
		Self {
			game,
			game_mode: Box::new(EmptyGameMode::new()),
			game_mode_initialized: false,
			levels: HashMap::new(),
			players: HashMap::new(),
			tick_multiplier: 1.0,
			needs_replication: true
		}
	}

	pub fn destroy(&mut self) {
		if self.game_mode_initialized {
			self.game_mode.on_end_play();
			self.game_mode_initialized = false;
		}
	}

	pub fn get_level_count(&self) -> usize {
		self.levels.len()
	}

	pub fn contains_level(&self, name: &String) -> bool {
		self.levels.get(name).is_some()
	}

	pub fn add_level(&mut self, name: &String) -> bool {
		if self.levels.contains_key(name) {
			return false;
		}
		let level = Level::new(self.game.clone(), name.clone());
		self.levels.insert(name.clone(), level);
		self.needs_replication = true;
		return true;
	}

	pub fn destroy_level(&mut self, name: &String) {
		if let Some(level) = self.levels.get_mut(name) {
			if level.active {
				level.on_end_play();
			}
			self.levels.remove_entry(name);
			self.needs_replication = true;
		}
	}

	pub fn activate_level(&mut self, name: &String) {
		let level = self.levels.get_mut(name);
		if let Some(level) = level {
			if !level.is_active() {

				// Only on activation, we need to check for success
				match level.on_begin_play() {
					Ok(_) => {},
					Err(error) => {
						println!("Error while initializing Level, giving up. Error: {}", error);
						return;
					}
				}
				level.active = true;
				self.needs_replication = true;
			}
		}
	}

	pub fn deactivate_level(&mut self, name: &String) {
		let level = self.levels.get_mut(name);
		if let Some(level) = level {
			if level.is_active() {
				level.on_end_play();
				level.active = false;
				self.needs_replication = true;
			}
		}
	}

	pub fn get_level(&self, name: &String) -> Option<&Level> {
		self.levels.get(&name.to_string())
	}

	pub fn get_level_mut(&mut self, name: &String) -> Option<&mut Level> {
		self.levels.get_mut(name)
	}

	pub fn set_game_mode(&mut self, game_mode: Box<dyn GameMode>) {
		if self.game_mode_initialized {
			self.game_mode.on_end_play();
		}
		self.game_mode_initialized = false;
		self.game_mode = game_mode;
	}

}