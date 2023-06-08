use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex};
use std::{sync};
use std::hash::{Hasher};
use spaghetti_engine_derive::Id;
use crate::core::Game;
use crate::events::event_listener::EventListener;
use crate::events::EventSource::{Client, Server};
use crate::events::{game_event, GameEvent};
use crate::utils::types::*;

pub struct EventDispatcher {
	game: sync::Weak<Game>,
	events: MutexVecDeque<EventRequest>,
	listeners: MutexHashMap<u64, Vec<ListenerEntry>>
}

impl EventDispatcher {

	pub fn new(game: sync::Weak<Game>) -> Self {
		Self {
			game,
			events: Mutex::new(VecDeque::new()),
			listeners: Mutex::new(HashMap::new()),
		}
	}

	pub fn raise_event(&self, mut event: Box<dyn GameEvent>, is_async: bool) {
		if let Some(game) = self.game.upgrade() {
			// Set the origin of the event
			event.get_event_data_mut().set_from(if game.is_client() { Client } else { Server });

			// Construct request
			let request = EventRequest::new(event);
			let request_id = request.request_id.clone();

			// Send the request
			self.events.lock().unwrap().push_back(request);

			// If async wait for completion
			let mut processing = is_async;
			while processing {
				let contains = self.events.lock()
					.unwrap()
					.iter()
					.position(|x| x.request_id == request_id);

				processing = contains.is_some();
			}
		}
	}

	fn dispatch_event(&self, request: &mut EventRequest) {
		let listener_map = self.listeners.lock().unwrap();

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
		let mut list = self.events.lock().unwrap();
		for event in list.iter_mut() {
			self.dispatch_event(event);
		}
	}

	pub fn register_event_listener<T: GameEvent + 'static>(&self, listener: Box<dyn EventListener>) -> Option<ListenerHandle> {
		let entry = ListenerEntry::new(listener);
		let entry_id = entry.entry_id.clone();
		let event_type = game_event::get_event_type::<T>();

		// Couldn't find that event type.
		// Should never happen unless the event type was not registered
		if event_type.is_none() {
			return None;
		}
		let event_type = event_type.unwrap();

		// Lock the map and attempt to retrieve listener list
		let mut map = self.listeners.lock().unwrap();
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

	pub fn unregister_event_listener(&self, id: ListenerHandle) {
		let mut map = self.listeners.lock().unwrap();
		for entry in map.iter_mut() {
			entry.1.retain(|x| x.entry_id != id);
		}
	}

}

// ListenerEntry
struct ListenerEntry {
	listener: Box<dyn EventListener>,
	entry_id: ListenerHandle
}
impl ListenerEntry {
	fn new(listener: Box<dyn EventListener>) -> Self {
		Self {
			listener,
			entry_id: ListenerHandle::new()
		}
	}
}

// EventRequest
struct EventRequest {
	event: Box<dyn GameEvent>,
	event_type: u64,
	request_id: EventRequestHandle
}



impl EventRequest {
	fn new(event: Box<dyn GameEvent>) -> Self {
		let event_type = event.get_event_type();
		Self {
			event,
			event_type,
			request_id: EventRequestHandle::new()
		}
	}
}

// ListenerHandle
#[derive(Id)]
#[bits64]
struct _ListenerHandle;

// EventRequestHandle
#[derive(Id)]
#[bits64]
struct _EventRequestHandle;