use crate::dispatcher::{FunctionDispatcher};

pub mod core;
pub mod demo;
pub mod dispatcher;
pub mod utils;
pub mod settings;
pub mod events;
pub mod networking;
pub mod world;
pub mod input;

pub fn main() {
    // Init dispatcher
    let dispatcher = FunctionDispatcher::from_current_thread();

    // Queue FUNCTION
    match dispatcher.queue_lambda_quick(|| {
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