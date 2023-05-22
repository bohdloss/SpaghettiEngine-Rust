use crate::world::{BeginEndPlay, Update};

pub trait GameMode : Update + BeginEndPlay { // TODO FIX PARAMETER TYPES
	fn on_client_join(&mut self, endpoint: i32, is_client: bool) -> i32; // endpoint is ConnectionEndpoint, returns a Controller
	fn on_client_leave(&mut self, endpoint: i32, is_client: bool);
	fn on_player_travel(&mut self, player_controller: i32, from: i32, to: i32, is_client: bool); // player_controller is PlayerController, from and to are Level
}