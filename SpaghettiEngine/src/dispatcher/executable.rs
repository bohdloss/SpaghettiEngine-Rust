use crate::dispatcher::DispatcherError;

pub trait Executable {
    fn execute(&self) -> Result<Option<u64>, DispatcherError>;
}

pub struct LambdaExecutable {
    pub(crate) function: fn() -> Result<Option<u64>, DispatcherError>
}

impl Executable for LambdaExecutable {
    fn execute(&self) -> Result<Option<u64>, DispatcherError> {
        let func = self.function;
        func()
    }
}

impl LambdaExecutable {
    pub const fn new(function: fn() -> Result<Option<u64>, DispatcherError>) -> Self {
        Self {
            function
        }
    }
}