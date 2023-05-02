use crate::dispatcher::{DispatcherError};
use std::collections::vec_deque::VecDeque;
use std::sync::{Mutex};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use crate::utils::ObjectPool;
use crate::utils::types::*;

pub struct FunctionDispatcher {
	handle_pool: ObjectPool<FunctionHandle, 256>,
    call_queue: Mutex<VecDeque<ArcRwLock<FunctionHandle>>>,
    thread_id: ThreadId,
	counter: u64
}

type FuncType = fn() -> Result<Option<u64>, DispatcherError>;

impl FunctionDispatcher {

	/// Constructs a new function dispatcher for the current thread id
	///
	/// # Returns
	///
	/// * The newly constructed dispatcher
	pub fn from_current_thread() -> FunctionDispatcher {
		Self::from_thread(thread::current().id())
	}

	/// Constructs a new function dispatcher for the given thread id
	///
	/// # Arguments
	///
	/// * `thread_id` - The id of the thread this dispatcher is owned by
	///
	/// # Returns
	///
	/// * The newly constructed dispatcher
    pub fn from_thread(thread_id: ThreadId) -> FunctionDispatcher {
        FunctionDispatcher {
	        handle_pool: ObjectPool::new(|| FunctionHandle::new()),
            call_queue: Mutex::new(VecDeque::new()),
            thread_id,
	        counter: 0
        }
    }

	/// Queues a function for the owning thread of this dispatcher to execute
	///
	/// # Arguments
	///
	/// * `function` - The function to queue
	///
	/// # Returns
	///
	/// * A handle to track the state of the request and obtain a return value.
	/// The handle must be freed with `free_handle()` once it is not needed anymore
    pub fn queue(&mut self, function: FuncType) -> ArcRwLock<FunctionHandle> {
        // ALLOCATE FROM OBJECT POOL
	    let pointer = self.handle_pool.borrow();
	    if let Ok(mut handle) = pointer.write() {
		    handle.id = {
			    let temp = self.counter;
			    self.counter += 1;
			    temp
		    };
		    handle.function = Some(function);
		    handle.return_value = None;
		    handle.finished = false;
	    } else {
		    panic!();
	    }

        // Is the current thread the one we are working for?
        if thread::current().id().eq(&self.thread_id) {
	        if let Ok(mut value) = pointer.write() {
		        value.process();
	        } else {
		        panic!();
	        }
        } else {
	        if let Ok(mut queue) = self.call_queue.lock() {
		        queue.push_back(pointer.clone());
	        } else {
		        panic!();
	        }
        }

	    pointer
    }

	/// Queues the given function, waits for the thread to execute it, then returns the output
	///
	/// # Arguments
	///
	/// * `function` - The function to queue
	///
	/// # Returns
	///
	/// * The return value of the function
	pub fn queue_quick(&mut self, function: FuncType) -> Result<Option<u64>, DispatcherError> {
		let handle = self.queue(function);
		let result;
		if let Ok(mut value) = handle.write() {
			result = value.get_return_value();
		} else {
			panic!();
		}
		self.free_handle(handle);
		result
	}

	/// Frees a handle returned by `queue()`
	///
	/// # Arguments
	///
	/// * `handle` - The handle
	pub fn free_handle(&mut self, handle: ArcRwLock<FunctionHandle>) {
		self.handle_pool.pay(handle);
	}

	/// Computes all queued functions. Only works on the owning thread
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
						panic!();
					}
				} else {
					break;
				}
			}
		} else {
			panic!();
		}
	}

}

pub struct FunctionHandle {
	id: u64,
    function: Option<FuncType>,
    return_value: Option<Result<Option<u64>, DispatcherError>>,
    finished: bool,
}

impl FunctionHandle {
    fn new() -> FunctionHandle {
        FunctionHandle {
	        id: 0,
            function: None,
            return_value: None,
            finished: true,
        }
    }

	/// Gets the return value of the queued function
	///
	/// # Returns
	///
	/// * The return value of the function, or an error if the function has not finished
	/// running yet. It is advised to call `wait_completion()` before this method
	pub fn get_return_value(&mut self) -> Result<Option<u64>, DispatcherError> {
		if !self.finished {
			return Err(DispatcherError::new(None, Some(String::from("Not finished yet"))));
		}
		if self.return_value.is_some() {
			return self.return_value.take().unwrap();
		}
		return Err(DispatcherError::new(None, Some(String::from("Return value not ready yet"))))
	}

	/// Waits for the function to finish running before returning
	pub fn wait_completion(&self) {
		while !self.finished {
			thread::sleep(Duration::from_millis(1));
		}
	}

	/// Gets the status of the function
	///
	/// # Returns
	///
	/// * Whether the function has finished
	pub fn has_finished(&self) -> bool {
		self.finished
	}

	/// Determines if the function generated a return value
	///
	/// # Returns
	///
	/// * Whether the function generated a return value
	pub fn has_return_value(&self) -> bool {
		if let Some(result) = &self.return_value {
			if let Ok(value) = result {
				return value.is_some();
			}
		}
		false
	}

	/// Determines if the function generated an error
	///
	/// # Returns
	///
	/// * Whether function generated an error
	pub fn has_error(&self) -> bool {
		if let Some(result) = &self.return_value {
			if let Err(_) = result {
				return true;
			}
		}
		false
	}

	/// Processes the function by running it and storing its return value
	pub fn process(&mut self) {
		if let Some(function) = &self.function {
			self.return_value = Some(function());
			self.finished = true;
		}
	}

}
