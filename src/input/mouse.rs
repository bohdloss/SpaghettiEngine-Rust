#[derive(Copy, Clone)]
pub enum Mouse {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
    LeftButton,
    RightButton,
    MiddleButton,
    Last,
}

impl Mouse {
    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn size() -> usize {
        Self::Last.index()
    }
}
