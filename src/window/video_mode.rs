use glfw::VidMode;

/// Represents a possible video mode for a monitor
pub struct VideoMode {
    pub(super) mode: VidMode,
}

impl VideoMode {
    /// # Returns
    /// * The size in pixels of this mode
    pub fn get_size(&self) -> (u32, u32) {
        (self.mode.width, self.mode.height)
    }

    /// # Returns
    /// * The refresh rate of this mode
    pub fn get_refresh_rate(&self) -> u32 {
        self.mode.refresh_rate
    }

    /// # Returns
    /// * The red, green, blue bit depths
    pub fn get_bit_depth(&self) -> (u32, u32, u32) {
        (
            self.mode.red_bits,
            self.mode.green_bits,
            self.mode.blue_bits
        )
    }
}
