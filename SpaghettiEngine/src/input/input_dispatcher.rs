use std::sync;
use crate::core::Game;

#[derive(Copy, Clone)]
enum MouseEventType {
    ButtonChange,
    Move,
    Scroll
}

pub struct InputDispatcher {
    game: sync::Weak<Game>
}