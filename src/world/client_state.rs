use std::sync;
use crate::input::controller::Controller;
use crate::world::GameObject;

pub struct ClientState {
    pub camera: sync::Weak<GameObject>,
    pub controller: sync::Weak<Controller>
}

impl ClientState {
    
    pub fn new() -> Self {
        Self {
            camera: sync::Weak::new(),
            controller: sync::Weak::new()
        }
    }
    
}