use std::error::Error;

pub struct DispatcherError {
    error: Option<&'static dyn Error>,
    message: Option<String>
}

impl DispatcherError {
    pub fn new(error: Option<&'static dyn Error>, message: Option<String>) -> DispatcherError {
        DispatcherError {
            error,
            message
        }
    }
}