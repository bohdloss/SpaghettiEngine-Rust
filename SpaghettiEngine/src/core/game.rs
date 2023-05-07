use crate::dispatcher::FunctionDispatcher;
use crate::events::event_dispatcher::EventDispatcher;

pub struct Game {
	event_dispatcher: EventDispatcher,
	primary_dispatcher: FunctionDispatcher,
	is_client: bool
}

impl Game {

	pub fn get_event_dispatcher(&self) -> &EventDispatcher {
		&self.event_dispatcher
	}

	pub fn get_primary_dispatcher(&self) -> &FunctionDispatcher {
		&self.primary_dispatcher
	}

	pub fn is_client(&self) -> bool {
		self.is_client
	}

}