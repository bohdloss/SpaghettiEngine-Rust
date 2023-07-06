use std::mem;

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum MouseAxis {
    X,
    Y,
    WheelX,
    WheelY,
    Unknown,
}

impl MouseAxis {
    pub const FIRST: MouseAxis = MouseAxis::X;
    pub const LAST: MouseAxis = MouseAxis::WheelY;

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
