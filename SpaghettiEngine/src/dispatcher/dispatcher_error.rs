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

    pub fn get_error(&self) -> Option<&'static dyn Error> {
        self.error.clone()
    }

    pub fn get_message(&self) -> Option<String> {
        self.message.clone()
    }
}