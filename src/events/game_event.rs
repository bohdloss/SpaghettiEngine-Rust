use crate::events::event_registry::EventType;
use crate::events::game_event::EventSource::*;
use crate::networking::replicate::Replicate;
use crate::utils::id_type::id_type;
use mopa::mopafy;
use std::hash::Hasher;

id_type!(EventId);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EventSource {
    NotSet,
    Client,
    Server,
}

pub struct EventData {
    id: EventId,
    from: EventSource,
    cancelled: bool,
}

impl EventData {
    pub fn new() -> Self {
        Self {
            id: EventId::new(),
            from: NotSet,
            cancelled: false,
        }
    }

    pub fn set_from(&mut self, from: EventSource) {
        if let NotSet = self.from {
            self.from = from;
        }
    }

    pub fn get_from(&self) -> EventSource {
        self.from
    }
    pub fn get_id(&self) -> &EventId {
        &self.id
    }
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

pub trait GameEvent: mopa::Any + Send {
    fn get_event_data(&self) -> &EventData;
    fn get_event_data_mut(&mut self) -> &mut EventData;
    fn get_event_type(&self) -> EventType;
}

mopafy!(GameEvent);

impl<T: GameEvent> Replicate for T {
    fn write_data(&self, _: bool) {
        unimplemented!();
    }

    fn read_data(&mut self, _: bool) {
        unimplemented!();
    }

    fn is_local(&self) -> bool {
        true
    }
}
