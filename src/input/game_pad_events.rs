use crate::events::{EventData, GameEvent};
use crate::register_game_event;

pub struct GamePadConnectEvent {
    data: EventData,
    pub game_pad_id: usize,
}

register_game_event!(GamePadConnectEvent, data -> data, new -> new_empty);

impl GamePadConnectEvent {
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

pub struct GamePadDisconnectEvent {
    data: EventData,
    pub game_pad_id: usize,
}

register_game_event!(GamePadDisconnectEvent, data -> data, new -> new_empty);

impl GamePadDisconnectEvent {
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
