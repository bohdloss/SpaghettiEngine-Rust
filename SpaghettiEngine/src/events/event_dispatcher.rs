use std::sync;
use std::sync::{Mutex, RwLock, RwLockReadGuard};
use crate::allocation::{object_pool, ObjectPool};
use crate::core::Game;
use crate::dispatcher::DispatcherError;
use crate::dispatcher::execute::Execute;
use crate::events::event_listener::EventListener;
use crate::events::EventSource::{Client, Server};
use crate::events::{GameEvent};
use crate::utils::types::*;

pub struct EventDispatcher {
	game: WeakRwLock<Game>,
	event_requests: Mutex<ObjectPool<RwLock<EventRequest>, { object_pool::DEFAULT_POOL_SIZE }>>,
	listeners: RwLockVec<Box<dyn EventListener>>
}

struct EventRequest {
	event: Option<Box<dyn GameEvent>>,
	game: WeakRwLock<Game>
}

impl EventRequest {

	fn new() -> Self {
		Self {
			event: None,
			game: sync::Weak::new()
		}
	}

}

impl Execute for EventRequest {
	fn execute(&mut self) -> DispatcherReturn {
		if let Some(game_ptr) = self.game.upgrade() {
			if let Some(event) = self.event.take() {

				// Simply dispatch the event
				let game = game_ptr.read().unwrap();
				game.get_event_dispatcher().dispatch_event(event);
				return Ok(None);
			}
		}
		Err(DispatcherError::new(None, Some(String::from("Either the Game or GameEvent pointer was null"))))
	}
}

impl EventDispatcher {

	pub fn new(game: WeakRwLock<Game>) -> Self {
		Self {
			game,
			event_requests: Mutex::new(ObjectPool::new(|| RwLock::new(EventRequest::new()))),
			listeners: RwLock::new(Vec::new())
		}
	}

	pub fn raise_event(&self, mut event: Box<dyn GameEvent>, is_async: bool) {
		if let Some(game_ptr) = self.game.upgrade() {
			let game = game_ptr.read().unwrap();
			// Set the origin of the event
			event.set_from(if game.is_client() { Client } else { Server });

			// Construct the request
			let request_ptr = self.event_requests.lock().unwrap().borrow();
			{
				let mut request = request_ptr.write().unwrap();

				request.event = Some(event);
				request.game = self.game.clone();
			}

			// Send the request
			if is_async {
				game.get_primary_dispatcher().queue(request_ptr);
			} else {
				game.get_primary_dispatcher().queue_quick(request_ptr).unwrap_or(None);
			}
		}
	}

	pub fn dispatch_event(&self, mut event: Box<dyn GameEvent>) {

	}

}