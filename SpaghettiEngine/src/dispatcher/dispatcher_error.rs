use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct DispatcherError {
    error: Option<&'static dyn Error>,
    message: Option<String>
}

impl Error for DispatcherError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error
    }
}

impl Debug for DispatcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self::Display::fmt(self, f)
    }
}

impl Display for DispatcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "Dispatcher error: {}", msg)?;
        } else {
            write!(f, "Dispatcher error.")?;
        }
        if let Some(err) = self.error {
            write!(f, "Caused by: {}", err)?;
        }
        Ok(())
    }
}

impl DispatcherError {
    pub fn new(error: Option<&'static dyn Error>, message: Option<&str>) -> DispatcherError {
        DispatcherError {
            error,
            message: match message {
                Some(string) => Some(string.to_string()),
                None => None
            }
        }
    }

    pub fn get_error(&self) -> Option<&'static dyn Error> {
        self.error.clone()
    }

    pub fn get_message(&self) -> Option<String> {
        self.message.clone()
    }
}