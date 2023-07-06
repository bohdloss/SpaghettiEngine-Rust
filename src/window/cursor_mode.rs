/// Represents a cursor mode
pub enum CursorMode {
    /// The mouse is hidden and cannot move outside the window
    Captured,
    /// The mouse is hidden but can move around as normal
    Invisible,
    /// The default cursor mode, the mouse is visible and movement is normal
    Normal
}