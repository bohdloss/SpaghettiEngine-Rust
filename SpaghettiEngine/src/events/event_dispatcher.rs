use std::collections::{HashMap, VecDeque};
use std::sync::{RwLock};
use std::{sync};
use crate::core::Game;
use crate::events::event_listener::EventListener;
use crate::events::EventSource::{Client, Server};
use crate::events::{game_event, GameEvent};
use crate::utils::id_provider;
use crate::utils::types::*;

pub struct EventDispatcher {
	game: sync::Weak<Game>,
	events: RwLockVecDeque<EventRequest>,
	listeners: RwLockHashMap<GenericId, Vec<ListenerEntry>>
}

struct ListenerEntry {
	listener: Box<dyn EventListener>,
	entry_id: GenericId // Used for de-registering a listener
}

impl ListenerEntry {
	fn new(listener: Box<dyn EventListener>) -> Self {
		Self {
			listener,
			entry_id: id_provider::generate_generic_id()
		}
	}
}

impl Drop for ListenerEntry {
	fn drop(&mut self) {
		id_provider::free_generic_id(self.entry_id);
	}
}

struct EventRequest {
	event: Box<dyn GameEvent>,
	event_type: GenericId,
	request_id: GenericId
}

impl EventRequest {
	fn new(event: Box<dyn GameEvent>) -> Self {
		let event_type = event.get_event_type();
		Self {
			event,
			event_type,
			request_id: id_provider::generate_generic_id()
		}
	}
}

impl Drop for EventRequest {
	fn drop(&mut self) {
		id_provider::free_generic_id(self.request_id);
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
			let request = EventRequest::new(event);
			let request_id = request.request_id;

			// Send the request
			self.events.write().unwrap().push_back(request);

			// If async wait for completion
			let mut processing = is_async;
			while processing {
				let contains = self.events.read()
					.unwrap()
					.iter()
					.position(|x| x.request_id == request_id);

				processing = contains.is_some();
			}
		}
	}

	fn dispatch_event(&self, request: &mut EventRequest) {
		let listener_map = self.listeners.read().unwrap();

		// Are there any listeners for this event type?
		if let Some(list) = listener_map.get(&request.event_type) {

			// Iterate over them and trigger them
			for listener_entry in list.iter() {
				listener_entry.listener.handle_event(
					self.game.clone(),
					&mut request.event
				);
			}
		}
	}

	pub fn process_events(&self) {
		let mut list = self.events.write().unwrap();
		for event in list.iter_mut() {
			self.dispatch_event(event);
		}
	}

	pub fn register_event_listener<T: GameEvent + 'static>(&self, listener: Box<dyn EventListener>) -> Option<GenericId> {
		let entry = ListenerEntry::new(listener);
		let entry_id = entry.entry_id;
		let event_type = game_event::get_event_id::<T>();

		// Couldn't find that event type.
		// Should never happen but we will still check for it
		if event_type.is_none() {
			return None;
		}
		let event_type = event_type.unwrap();

		// Lock the map and attempt to retrieve listener list
		let mut map = self.listeners.write().unwrap();
		let mut list = map.get_mut(&event_type);

		// First time registering a listener of this kind
		if list.is_none() {
			map.insert(event_type, Vec::new());
			list = map.get_mut(&event_type);
		}
		let list = list.unwrap();

		// Add listener to list
		list.push(entry);

		Some(entry_id)
	}

	pub fn unregister_event_listener(&self, id: GenericId) {
		let mut map = self.listeners.write().unwrap();
		for entry in map.iter_mut() {
			entry.1.retain(|x| x.entry_id != id);
		}
	}

}