use crate::events::{EventData, GameEvent};
use crate::register_game_event;

pub struct GamePadConnectedEvent {
    data: EventData,
    pub game_pad_id: usize,
}

register_game_event!(GamePadConnectedEvent, data -> data, new -> new_empty);

impl GamePadConnectedEvent {
    pub fn new_empty() -> Box<dyn GameEvent> {
        Box::new(Self {
            data: EventData::new(),
            game_pad_id: 0,
        })
    }

    pub fn new(game_pad_id: usize) -> Box<dyn GameEvent> {
        Box::new(Self {
            data: EventData::new(),
            game_pad_id,
        })
    }
}

pub struct GamePadDisconnectedEvent {
    data: EventData,
    pub game_pad_id: usize,
}

register_game_event!(GamePadDisconnectedEvent, data -> data, new -> new_empty);

impl GamePadDisconnectedEvent {
    pub fn new_empty() -> Box<dyn GameEvent> {
        Box::new(Self {
            data: EventData::new(),
            game_pad_id: 0,
        })
    }

    pub fn new(game_pad_id: usize) -> Box<dyn GameEvent> {
        Box::new(Self {
            data: EventData::new(),
            game_pad_id,
        })
    }
}
