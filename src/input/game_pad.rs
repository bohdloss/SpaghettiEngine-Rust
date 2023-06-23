#[derive(Copy, Clone)]
pub enum GamePad {
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
    Last,
}

impl GamePad {
    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn size() -> usize {
        Self::Last.index()
    }
}

#[derive(Copy, Clone)]
pub enum GamePadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
    Last,
}

impl GamePadAxis {
    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn size() -> usize {
        Self::Last.index()
    }
}
