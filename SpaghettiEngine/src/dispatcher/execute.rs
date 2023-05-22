use crate::utils::types::*;

pub trait Execute {
	fn execute(&mut self) -> DispatcherReturn;
}