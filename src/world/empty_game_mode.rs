use crate::utils::types::float;
use crate::world::{BeginEndPlay, BeginError, GameMode, Update};

pub struct EmptyGameMode {}

impl EmptyGameMode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Update for EmptyGameMode {
    fn update(&mut self, _delta: float) {}
}

impl BeginEndPlay for EmptyGameMode {
    fn on_begin_play(&mut self) -> Result<(), BeginError> {
        Ok(())
    }

    fn on_end_play(&mut self) {}
}

impl GameMode for EmptyGameMode {
    fn on_client_join(&mut self, _endpoint: i32, _is_client: bool) -> i32 {
        0
    }

    fn on_client_leave(&mut self, _endpoint: i32, _is_client: bool) {}

    fn on_player_travel(
        &mut self,
        _player_controller: i32,
        _from: i32,
        _to: i32,
        _is_client: bool,
    ) {
    }
}
