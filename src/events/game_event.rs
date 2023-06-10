use std::any::{TypeId};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::events::game_event::EventSource::*;
use crate::id_type;
use crate::networking::replicate::Replicate;
use crate::utils::as_any::AsAny;
use crate::utils::types::*;

id_type!(EventId);

#[derive(Copy, Clone)]
pub enum EventSource {
	NotSet,
	Client,
	Server
}

pub struct EventData {
	id: EventId,
	from: EventSource,
	cancelled: bool
}

impl EventData {
	pub fn new() -> Self {
		Self {
			id: EventId::new(),
			from: NotSet,
			cancelled: false
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

pub trait GameEvent : Send + AsAny {
	fn get_event_data(&self) -> &EventData;
	fn get_event_data_mut(&mut self) -> &mut EventData;
	fn get_event_type(&self) -> u64;
}

impl<T: GameEvent> Replicate for T {
	fn write_data_server(&self) {
		unimplemented!();
	}

	fn read_data_server(&self) {
		unimplemented!();
	}

	fn write_data_client(&self) {
		unimplemented!();
	}

	fn read_data_client(&self) {
		unimplemented!();
	}

	fn is_local(&self) -> bool {
		true
	}
}

pub type EventConstructor = fn() -> Box<dyn GameEvent>;

static EVENT_TABLE: Lazy<RwLockHashMap<u64, EventConstructor>> = Lazy::new(|| RwLock::new(HashMap::new()));
static TYPE_ASSOCIATION_TABLE: Lazy<RwLockHashMap<TypeId, u64>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_event_type<T: GameEvent + 'static>(constructor: EventConstructor) -> u64 {
	let mut event_table = EVENT_TABLE.write().unwrap();

	// Generate an id for the event
	let mut hasher = DefaultHasher::new();
	stringify!(T).hash(&mut hasher);
	let id = hasher.finish();

	// Add to the table
	event_table.insert(id, constructor);

	// Register to type association table
	let mut association_table = TYPE_ASSOCIATION_TABLE.write().unwrap();
	association_table.insert(TypeId::of::<T>(), id);
	id
}

pub fn get_event_constructor(event_type: u64) -> Option<EventConstructor> {
	let event_table = EVENT_TABLE.read().unwrap();
	match event_table.get(&event_type) {
		Some(func) => Some(*func),
		None => None
	}
}

pub fn get_event_type<T: GameEvent + 'static>() -> Option<u64> {
	let association_table = TYPE_ASSOCIATION_TABLE.read().unwrap();
	match association_table.get(&TypeId::of::<T>()) {
		Some(id) => Some(*id),
		None => None
	}
}