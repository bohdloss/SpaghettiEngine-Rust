use std::fmt::{Debug, Display, Formatter};
use std::mem;

impl Display for MouseButton {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseButton {
    LeftButton,
    RightButton,
    MiddleButton,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
    Unknown,
}

impl MouseButton {
    pub const FIRST: MouseButton = MouseButton::LeftButton;
    pub const LAST: MouseButton = MouseButton::Button8;

    pub fn from_usize(idx: usize) -> Self {
        if idx < Self::size() {
            unsafe { mem::transmute(idx as u8) }
        } else {
            Self::Unknown
        }
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn size() -> usize {
        Self::LAST.index() + 1
    }
}

impl Display for MouseAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseAxis {
    Position,
    Wheel,
    Unknown,
}

impl MouseAxis {
    pub const FIRST: MouseAxis = MouseAxis::Position;
    pub const LAST: MouseAxis = MouseAxis::Wheel;

    pub fn from_usize(idx: usize) -> Self {
        if idx < Self::size() {
            unsafe { mem::transmute(idx as u8) }
        } else {
            Self::Unknown
        }
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn size() -> usize {
        Self::LAST.index() + 1
    }
}
