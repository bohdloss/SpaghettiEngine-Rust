use crate::utils::types::float;
use crate::world::{BeginEndPlay, BeginError, Update};

pub struct Level {
    name: String,
    pub(super) active: bool,
}

impl Update for Level {
    fn update(&mut self, delta: float) {}
}

impl BeginEndPlay for Level {
    fn on_begin_play(&mut self) -> Result<(), BeginError> {
        Ok(())
    }

    fn on_end_play(&mut self) {}
}

impl Level {
    pub fn new(name: String) -> Self {
        Self {
            name,
            active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}
