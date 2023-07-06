use std::sync::Mutex;

pub trait IsLocked {
    fn is_locked(&self) -> bool;
}

impl<T> IsLocked for Mutex<T> {
    fn is_locked(&self) -> bool {
        match self.try_lock() {
            Ok(_) => false,
            Err(_) => true
        }
    }
}