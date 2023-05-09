use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::{sync, thread};
use std::time::Duration;
use crate::core::Game;
use crate::events::event_listener::EventListener;
use crate::events::EventSource::{Client, Server};
use crate::events::{GameEvent};
use crate::utils::id_provider;
use crate::utils::types::*;

pub struct EventDispatcher {
	game: sync::Weak<Game>,
	events: RwLockVecDeque<Arc<EventRequest>>,
	listeners: RwLockHashMap<ObjectId, Vec<ListenerEntry>>
}

struct ListenerEntry {
	listener: Box<dyn EventListener>,
	id: GenericId
}

impl ListenerEntry {
	fn new(listener: Box<dyn EventListener>) -> Self {
		Self {
			listener,
			id: id_provider::generate_generic_id()
		}
	}
}

impl Drop for ListenerEntry {
	fn drop(&mut self) {
		id_provider::free_generic_id(self.id);
	}
}

struct EventRequest {
	event: Box<dyn GameEvent>,
	completed: bool
}

impl EventRequest {
	fn new(event: Box<dyn GameEvent>) -> Self {
		Self {
			event,
			completed: false
		}
	}
}

impl EventDispatcher {

	pub fn new(game: sync::Weak<Game>) -> Self {
		Self {
			game,
			events: RwLock::new(VecDeque::new()),
			listeners: RwLock::new(HashMap::new()),
		}
	}

	pub fn raise_event(&self, mut event: Box<dyn GameEvent>, is_async: bool) {
		if let Some(game) = self.game.upgrade() {
			// Set the origin of the event
			event.set_from(if game.is_client() { Client } else { Server });

			// Construct request
			let request = Arc::new(EventRequest::new(event));

			// Send the request
			self.events.write().unwrap().push_back(request.clone());

			// If async wait for completion
			while is_async && !request.completed {
				thread::sleep(Duration::from_millis(1));
			}
		}
	}

	fn dispatch_event(&self, mut event: Box<dyn GameEvent>) {
		if let Some(game) = self.game.upgrade() {
			// Override origin of event (Note: this is inverted on purpose)
			event.set_from(if game.is_client() { Server } else { Client });

			let listeners = self.listeners.read().unwrap();
			listeners.iter().for_each(|x| {

			});
		}
	}

	pub fn process_events(&self) {

	}

	pub fn register_event_listener(&self, listener: Box<dyn EventListener>) -> GenericId {
		let entry = ListenerEntry::new(listener);
		let id = entry.id;
		//self.listeners.write().unwrap().push(entry);
		id
	}

	pub fn unregister_event_listener(&self, id: GenericId) {
		let mut list = self.listeners.write().unwrap();
		/*let contains = list.iter()
			.position(|x| x.id == id);

		if let Some(index) = contains {
			list.remove(index);
		}*/
	}

}