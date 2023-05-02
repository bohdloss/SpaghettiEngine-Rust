use crate::dispatcher::{DispatcherError, Executable};
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use crate::utils::types::*;

pub struct FunctionDispatcher {
    call_queue: Mutex<VecDeque<ArcRwLock<FunctionHandle>>>,
    thread_id: ThreadId,
	counter: u64
}

impl FunctionDispatcher {

	pub fn from_current_thread() -> FunctionDispatcher {
		FunctionDispatcher {
			call_queue: Mutex::new(VecDeque::new()),
			thread_id: thread::current().id(),
			counter: 0,
		}
	}

    pub fn from_thread(thread_id: ThreadId) -> FunctionDispatcher {
        FunctionDispatcher {
            call_queue: Mutex::new(VecDeque::new()),
            thread_id,
	        counter: 0
        }
    }

    fn on_poison_error() {
        println!("Poison error while accessing FUNCTION queue");
        panic!();
    }

	pub fn queue_with_return(&mut self, function: &'static dyn Executable) -> Arc<RwLock<FunctionHandle>> {
		self.queue(function, false)
	}

    pub fn queue(&mut self, function: &'static dyn Executable, ignore_return_value: bool) -> Arc<RwLock<FunctionHandle>> {
        // Initialize struct
	    let mut handle = FunctionHandle::new();
	    handle.id = {let temp = self.counter; self.counter += 1; temp};
        handle.function = Some(function);
        handle.return_value = None;
        handle.error = None;
        handle.ignore_return_value = ignore_return_value;
        handle.finished = false;

	    let pointer = Arc::new(RwLock::new(handle));

        // Is the current thread the one we are working for?
        if thread::current().id().eq(&self.thread_id) {
	        if let Ok(mut value) = pointer.write() {
		        value.process();
	        } else {
		        Self::on_poison_error();
	        }
        } else {
	        if let Ok(mut queue) = self.call_queue.lock() {
		        queue.push_back(pointer.clone());
	        } else {
		        Self::on_poison_error();
	        }
        }

	    pointer
    }

	pub fn queue_quick(&mut self,
		function: &'static dyn Executable
	) -> Result<Option<u64>, DispatcherError> {
		let handle = self.queue_with_return(function);
		if let Ok(mut value) = handle.write() {
			return  value.get_return_value();
		} else {
			Self::on_poison_error();
		}
		Err(DispatcherError::new(None, Some(String::from("Handle poisoned"))))
	}

	pub fn compute_tasks(&mut self) {
		if !thread::current().id().eq(&self.thread_id) {
			return;
		}
		if let Ok(mut queue) = self.call_queue.lock() {
			loop {
				if let Some(locked) = queue.pop_front() {
					if let Ok(mut handle) = locked.write() {
						handle.process();
					} else {
						Self::on_poison_error();
					}
				} else {
					break;
				}
			}
		} else {
			Self::on_poison_error();
		}
	}

}

pub struct FunctionHandle {
	id: u64,
    function: Option<&'static dyn Executable>,
    return_value: Option<u64>,
    error: Option<DispatcherError>,
    ignore_return_value: bool,
    finished: bool,
}

impl FunctionHandle {
    pub fn new() -> FunctionHandle {
        FunctionHandle {
	        id: 0,
            function: None,
            return_value: None,
            error: None,
            ignore_return_value: false,
            finished: false,
        }
    }

	pub fn get_return_value(&mut self) -> Result<Option<u64>, DispatcherError> {
		if !self.finished {
			return Err(DispatcherError::new(None, Some(String::from("Not finished yet"))));
		}
		if self.has_exception() {
			Err(self.error.take().unwrap())
		} else {
			Ok(self.return_value)
		}
	}

	pub fn wait_return_value(&self) {
		while !self.finished && !self.ignore_return_value {
			thread::sleep(Duration::from_millis(1));
		}
	}

	pub fn wait_completion(&self) {
		while !self.finished {
			thread::sleep(Duration::from_millis(1));
		}
	}

	pub fn has_finished(&self) -> bool {
		self.finished
	}

	pub fn has_return_value(&self) -> bool {
		self.return_value.is_some()
	}

	pub fn has_exception(&self) -> bool {
		self.error.is_some()
	}

	pub fn process(&mut self) {
		if self.function.is_none() {
			return;
		}
		match self.function.unwrap().execute() {
			Ok(result) => {
				self.return_value = result;
				self.error = None;
			},
			Err(error) => {
				self.return_value = None;
				self.error = Some(error);
			}
		}
		self.finished = true;
	}

}
