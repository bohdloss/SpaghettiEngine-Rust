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
	id: IdType,
	from: EventSource,
	cancelled: bool
}

impl EventData {
	pub fn new() -> Self {
		Self {
			id: id_provider::generate_id(),
			from: NotSet,
			cancelled: false
		}
	}
}

impl Drop for EventData {
	fn drop(&mut self) {
		id_provider::free_id(self.id)
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
	fn get_id(&self) -> IdType {
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