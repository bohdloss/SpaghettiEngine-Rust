use std::any::{TypeId};
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::events::game_event::EventSource::*;
use crate::networking::replicate::Replicate;
use crate::utils::as_any::AsAny;
use crate::utils::id_provider;
use crate::utils::types::*;

#[derive(Copy, Clone)]
pub enum EventSource {
	NotSet,
	Client,
	Server
}

pub struct EventData {
	id: ObjectId,
	from: EventSource,
	cancelled: bool
}

impl EventData {
	pub fn new() -> Self {
		Self {
			id: id_provider::generate_object_id(),
			from: NotSet,
			cancelled: false
		}
	}
}

impl Drop for EventData {
	fn drop(&mut self) {
		id_provider::free_object_id(self.id)
	}
}

pub trait GameEvent : AsAny {
	fn set_from(&mut self, from: EventSource) {
		let data = self.get_event_data_mut();
		if let NotSet = data.from {
			data.from = from;
		}
	}
	fn get_from(&self) -> EventSource {
		self.get_event_data().from
	}
	fn get_id(&self) -> ObjectId {
		self.get_event_data().id
	}
	fn set_cancelled(&mut self, cancelled: bool) {
		self.get_event_data_mut().cancelled = cancelled;
	}
	fn is_cancelled(&self) -> bool {
		self.get_event_data().cancelled
	}
	fn get_event_data(&self) -> &EventData;
	fn get_event_data_mut(&mut self) -> &mut EventData;
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

static EVENT_TABLE: RwLockVec<fn() -> Box<dyn GameEvent>> = RwLock::new(Vec::new());
static TYPE_ASSOCIATION_TABLE: Lazy<RwLockHashMap<TypeId, GenericId>> = Lazy::new(|| RwLock::new(HashMap::new()));

// NOT QUITE
// Use an event_table file in the future
pub fn register_event_type<T: GameEvent + 'static>(constructor: fn() -> Box<dyn GameEvent>) -> GenericId {
	let mut event_table = EVENT_TABLE.write().unwrap();
	let id = event_table.len() as GenericId;
	event_table.push(constructor);
	let mut association_table = TYPE_ASSOCIATION_TABLE.write().unwrap();
	association_table.insert(TypeId::of::<T>(), id);
	id
}

pub fn get_event_constructor(id: GenericId) -> Option<fn() -> Box<dyn GameEvent>> {
	let event_table = EVENT_TABLE.read().unwrap();
	match event_table.get(id as usize) {
		Some(func) => Some(*func),
		None => None
	}
}

pub fn get_event_id<T: GameEvent + 'static>() -> Option<GenericId> {
	let association_table = TYPE_ASSOCIATION_TABLE.read().unwrap();
	match association_table.get(&TypeId::of::<T>()) {
		Some(id) => Some(*id),
		None => None
	}
}