use crate::dispatcher::{FunctionDispatcher};

mod core;
mod demo;
mod dispatcher;
mod utils;

fn main() {
    // Init dispatcher
    let mut dispatcher = FunctionDispatcher::from_current_thread();

    // Queue FUNCTION
    match dispatcher.queue_quick(|| {
        println!("HAIIII :3");
        Ok(Some(4815162342))
    }) {
        Ok(value) => {
            if let Some(val) = value {
                println!("{val}");
            } else {
                println!("No return value")
            }
        },
        Err(error) => {
            let msg = error.get_message();
            if let Some(str) = msg {
                println!("Dispatcher error: {str}");
            } else {
                println!("Dispatcher error");
            }
        }
    }
}
