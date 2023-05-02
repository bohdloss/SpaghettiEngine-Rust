use crate::dispatcher::{DispatcherError, Executable, FunctionDispatcher};
use std::thread;

mod core;
mod demo;
mod dispatcher;
mod utils;

struct TestFunction {
}

impl Executable for TestFunction {
    fn execute(&self) -> Result<Option<u64>, DispatcherError> {
        println!("hello from test FUNCTION!");
        Ok(Some(89))
    }
}

static FUNCTION: TestFunction = TestFunction{};

fn main() {
    thread::spawn(|| {});
    // Init dispatcher
    let mut dispatcher = FunctionDispatcher::from_current_thread();

    // Queue FUNCTION
    match dispatcher.queue_quick(&FUNCTION) {
        Ok(value) => {
            if value.is_some() {
                let vall = value.unwrap();
                println!("{vall}");
            } else {
                println!("No return value")
            }
        },
        Err(_) => {
            println!("Dispatcher error");
        }
    }
}
