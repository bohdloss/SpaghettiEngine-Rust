use crate::events::game_event::*;
use crate::register_game_event;

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
pub struct NothingHappenedEvent {
    event_data: EventData,
}

register_game_event!(NothingHappenedEvent, data -> event_data, new -> new_empty);

impl NothingHappenedEvent {
    pub fn new_empty() -> Box<dyn GameEvent> {
        Box::new(Self {
            event_data: EventData::new(),
        })
    }
}
