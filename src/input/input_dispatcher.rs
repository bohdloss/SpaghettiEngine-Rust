use crate::core::Game;
use std::sync;

#[derive(Copy, Clone)]
enum MouseEventType {
    ButtonChange,
    Move,
    Scroll,
}

pub struct InputDispatcher {
    game: sync::Weak<Game>,
}
