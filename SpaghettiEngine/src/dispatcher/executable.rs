use crate::dispatcher::DispatcherError;

pub trait Executable {
    fn execute(&self) -> Result<Option<u64>, DispatcherError>;
}
