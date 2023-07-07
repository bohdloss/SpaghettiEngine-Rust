use std::mem;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GamePadButton {
    Cross,
    Circle,
    Square,
    Triangle,
    LeftBumper,
    RightBumper,
    Back,
    Start,
    Guide,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadRight,
    DPadDown,
    DPadLeft,
    Unknown,
}

impl GamePadButton {
    pub const FIRST: GamePadButton = GamePadButton::Cross;
    pub const LAST: GamePadButton = GamePadButton::DPadLeft;

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
pub enum GamePadAxis {
    LeftThumbStick,
    RightThumbStick,
    LeftRightTriggers,
    Unknown,
}

impl GamePadAxis {
    pub const FIRST: GamePadAxis = GamePadAxis::LeftThumbStick;
    pub const LAST: GamePadAxis = GamePadAxis::LeftRightTriggers;

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
