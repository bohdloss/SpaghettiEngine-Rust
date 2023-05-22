use crate::dispatcher::{DispatcherError};
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use crate::dispatcher::execute::{Execute};
use crate::utils::types::*;

pub struct FunctionDispatcher {
    call_queue: MutexVecDeque<ArcRwLock<FunctionHandle>>,
    thread_id: ThreadId
}

impl FunctionDispatcher {

	/// Constructs a new function dispatcher for the current thread id
	///
	/// # Returns
	///
	/// * The newly constructed dispatcher
	pub fn from_current_thread() -> Self {
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
    pub fn from_thread(thread_id: ThreadId) -> Self {
        Self {
            call_queue: Mutex::new(VecDeque::new()),
            thread_id
        }
    }

	pub fn queue_lambda(&self, function: DispatcherFunction) -> ArcRwLock<FunctionHandle> {
		self.queue(Box::new(LambdaExecute::new(function)))
	}

	/// Queues a function for the owning thread of this dispatcher to execute
	///
	/// # Arguments
	/// * `function` - The function to queue
	///
	/// # Returns
	/// * A handle to track the state of the request and obtain a return value.
    pub fn queue(&self, function: Box<dyn Execute>) -> ArcRwLock<FunctionHandle> {
		let handle = Arc::new(RwLock::new(FunctionHandle::new(function)));

        // Is the current thread the one we are working for?
        if thread::current().id().eq(&self.thread_id) {
	        {
		        let mut value = handle.write().unwrap();
		        value.process();
	        }
        } else {
	        {
		        let mut queue = self.call_queue.lock().unwrap();
		        queue.push_back(handle.clone());
	        }
        }

		handle
    }

	pub fn queue_lambda_quick(&self, function: DispatcherFunction) -> DispatcherReturn {
		self.queue_quick(Box::new(LambdaExecute::new(function)))
	}

	/// Queues the given function, waits for the thread to execute it, then returns the output
	///
	/// # Arguments
	/// * `function` - The function to queue
	///
	/// # Returns
	/// * The return value of the function
	pub fn queue_quick(&self, function: Box<dyn Execute>) -> DispatcherReturn {
		let handle = self.queue(function);
		let result;
		{
			let mut value = handle.write().unwrap();
			result = value.get_return_value();
		}
		result
	}

	/// Computes all queued functions. Must only be called on the owning thread
	pub fn compute_tasks(&self) {
		loop {
			let mut queue = self.call_queue.lock().unwrap();
			if let Some(locked) = queue.pop_front() {
					locked.write().unwrap().process();
			} else { break; }
		}
	}

}

pub struct FunctionHandle {
    function: Box<dyn Execute>,
    return_value: Option<Result<Option<u64>, DispatcherError>>,
    finished: bool,
}

impl FunctionHandle {
    fn new(function: Box<dyn Execute>) -> FunctionHandle {
        FunctionHandle {
            function,
            return_value: None,
            finished: false,
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
			return Err(DispatcherError::new(None, Some("Not finished yet")));
		}
		if self.return_value.is_some() {
			return self.return_value.take().unwrap();
		}
		return Err(DispatcherError::new(None, Some("Return value not ready yet")))
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
		self.return_value = Some(self.function.execute());
		self.finished = true;
	}

}

struct LambdaExecute {
	function: DispatcherFunction
}

impl LambdaExecute {

	pub fn new(function: DispatcherFunction) -> Self {
		Self {
			function
		}
	}

}

impl Execute for LambdaExecute {
	fn execute(&mut self) -> DispatcherReturn {
		(self.function)()
	}

}