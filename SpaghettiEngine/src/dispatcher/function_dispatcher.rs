use crate::dispatcher::{DispatcherError};
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use crate::allocation::{object_pool, ObjectPool};
use crate::dispatcher::execute::{Execute};
use crate::utils::types::*;

pub struct FunctionDispatcher {
	handle_pool: Mutex<ObjectPool<RwLock<FunctionHandle>, { object_pool::DEFAULT_POOL_SIZE }>>,
    lambda_pool: ArcMutex<ObjectPool<RwLock<LambdaExecute>, { object_pool::DEFAULT_POOL_SIZE }>>,
    call_queue: RwLockVecDeque<ArcRwLock<FunctionHandle>>,
    thread_id: ThreadId
}

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
	        handle_pool: Mutex::new(ObjectPool::new(|| RwLock::new(FunctionHandle::new()))),
	        lambda_pool: Arc::new(Mutex::new(ObjectPool::new(|| RwLock::new(LambdaExecute::new())))),
            call_queue: RwLock::new(VecDeque::new()),
            thread_id
        }
    }

	pub fn queue_lambda(&self, function: DispatcherFunction) -> ArcRwLock<FunctionHandle> {
		let object = self.lambda_pool.lock().unwrap().borrow();
		{
			let mut lambda = object.write().unwrap();
			lambda.function = function;
			lambda.self_ptr = Arc::downgrade(&object);
			lambda.pool = Arc::downgrade(&self.lambda_pool);
		}
		self.queue(object)
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
    pub fn queue(&self, function: ArcRwLock<dyn Execute>) -> ArcRwLock<FunctionHandle> {
        // ALLOCATE FROM OBJECT POOL
	    let pointer = self.handle_pool.lock().unwrap().borrow();
		{
			let mut handle = pointer.write().unwrap();
			handle.function = Some(function);
			handle.return_value = None;
			handle.finished = false;
		}

        // Is the current thread the one we are working for?
        if thread::current().id().eq(&self.thread_id) {
	        {
		        let mut value = pointer.write().unwrap();
		        value.process();
	        }
        } else {
	        {
		        let mut queue = self.call_queue.write().unwrap();
		        queue.push_back(pointer.clone());
	        }
        }

	    pointer
    }

	pub fn queue_lambda_quick(&self, function: DispatcherFunction) -> DispatcherReturn {
		let object = self.lambda_pool.lock().unwrap().borrow();
		{
			let mut lambda = object.write().unwrap();
			lambda.function = function;
			lambda.self_ptr = Arc::downgrade(&object);
			lambda.pool = Arc::downgrade(&self.lambda_pool.clone());
		}
		self.queue_quick(object)
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
	pub fn queue_quick(&self, function: ArcRwLock<dyn Execute>) -> DispatcherReturn {
		let handle = self.queue(function);
		let result;
		{
			let mut value = handle.write().unwrap();
			result = value.get_return_value();
		}
		self.free_handle(handle);
		result
	}

	/// Frees a handle returned by `queue()`
	///
	/// # Arguments
	///
	/// * `handle` - The handle
	pub fn free_handle(&self, handle: ArcRwLock<FunctionHandle>) {
		self.handle_pool.lock().unwrap().pay(handle);
	}

	/// Computes all queued functions. Must only be called on the owning thread
	pub fn compute_tasks(&self) {
		loop {
			let mut queue = self.call_queue.write().unwrap();
			if let Some(locked) = queue.pop_front() {
					locked.write().unwrap().process();
			} else { break; }
		}
	}

}

pub struct FunctionHandle {
    function: Option<ArcRwLock<dyn Execute>>,
    return_value: Option<Result<Option<u64>, DispatcherError>>,
    finished: bool,
}

impl FunctionHandle {
    fn new() -> FunctionHandle {
        FunctionHandle {
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
			self.return_value = Some(function.write().unwrap().execute());
			self.finished = true;
		}
	}

}

struct LambdaExecute {
	function: DispatcherFunction,
	pool: WeakMutex<ObjectPool<RwLock<LambdaExecute>, { object_pool::DEFAULT_POOL_SIZE }>>,
	self_ptr: WeakRwLock<LambdaExecute>
}

impl LambdaExecute {

	pub fn new() -> Self {
		Self {
			function: || Ok(None),
			pool: Weak::new(),
			self_ptr: Weak::new()
		}
	}

}

impl Execute for LambdaExecute {
	fn execute(&mut self) -> DispatcherReturn {
		let result = (self.function)();

		// Make this object reusable in the pool
		if let Some(pool) = self.pool.upgrade() {
			if let Some(self_ptr) = self.self_ptr.upgrade() {
				pool.lock().unwrap().pay(self_ptr);
			}
		}
		result
	}

}