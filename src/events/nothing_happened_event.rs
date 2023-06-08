use spaghetti_engine_derive::{AsAny, GameEvent};
use crate::events::game_event::*;
use crate::events::game_event;
use crate::utils::AsAny;

/// ░░░░░▄▄▄▄▀▀▀▀▀▀▀▀▄▄▄▄▄▄░░░░░░░░<br>
/// ░░░░░█░░░░▒▒▒▒▒▒▒▒▒▒▒▒░░▀▀▄░░░░<br>
/// ░░░░█░░░▒▒▒▒▒▒░░░░░░░░▒▒▒░░█░░░<br>
/// ░░░█░░░░░░▄██▀▄▄░░░░░▄▄▄░░░░█░░<br>
/// ░▄▀▒▄▄▄▒░█▀▀▀▀▄▄█░░░██▄▄█░░░░█░<br>
/// █░▒█▒▄░▀▄▄▄▀░░░░░░░░█░░░▒▒▒▒▒░█<br>
/// █░▒█░█▀▄▄░░░░░█▀░░░░▀▄░░▄▀▀▀▄▒█<br>
/// ░█░▀▄░█▄░█▀▄▄░▀░▀▀░▄▄▀░░░░█░░█░<br>
/// ░░█░░░▀▄▀█▄▄░█▀▀▀▄▄▄▄▀▀█▀██░█░░<br>
/// ░░░█░░░░██░░▀█▄▄▄█▄▄█▄████░█░░░<br>
/// ░░░░█░░░░▀▀▄░█░░░█░█▀██████░█░░<br>
/// ░░░░░▀▄░░░░░▀▀▄▄▄█▄█▄█▄█▄▀░░█░░<br>
/// ░░░░░░░▀▄▄░▒▒▒▒░░░░░░░░░░▒░░░█░<br>
/// ░░░░░░░░░░▀▀▄▄░▒▒▒▒▒▒▒▒▒▒░░░░█░<br>
/// ░░░░░░░░░░░░░░▀▄▄▄▄▄░░░░░░░░█░░<br>
#[derive(GameEvent, AsAny)]
pub struct NothingHappenedEvent {
	event_data: EventData
}

impl NothingHappenedEvent {
	pub fn new_empty() -> Self {
		Self {
			event_data: EventData::new()
		}
	}

	pub fn new() -> Self {
		Self {
			event_data: EventData::new()
		}
	}
}