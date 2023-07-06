use crate::window::video_mode::VideoMode;
use glfw::Monitor;
use crate::window::window_manager::*;
use crate::window::packets::*;

macro_rules! glfw_request {
    ($packet:ident ( $($request_args:expr),+ ), $response:ident ( $($response_args:ident),+ ), $ret:expr) => {{
        match GLFW.request(GlfwPacket::$packet( $($request_args),* )) {
            GlfwPacket::$response ( $($response_args),* ) => {Ok($ret)},
            GlfwPacket::GError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($packet:ident ( $($request_args:expr),+ ), $response:ident, $ret:expr) => {{
        match GLFW.request(GlfwPacket::$packet( $($request_args),* )) {
            GlfwPacket::$response => {Ok($ret)},
            GlfwPacket::GError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($packet:ident, $response:ident ( $($response_args:ident),+ ), $ret:expr) => {{
        match GLFW.request(GlfwPacket::$packet) {
            GlfwPacket::$response ( $($response_args),* ) => {Ok($ret)},
            GlfwPacket::GError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($packet:ident, $response:ident, $ret:expr) => {{
        match GLFW.request(GlfwPacket::$packet) {
            GlfwPacket::$response => {Ok($ret)},
            GlfwPacket::GError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
}

/// Represents a window monitor and provides
/// some utility functions
pub struct WindowMonitor<'a> {
    pub(super) monitor: &'a mut Monitor,
}

impl<'a> WindowMonitor<'a> {
    /// Allows you to iterate over monitors and perform
    /// operations on them
    ///
    /// # Arguments
    /// * `f` - The function to receive the monitor list
    ///
    /// # Returns
    /// * Whatever `f` returns
    pub fn with_monitors<T, R>(f: T) -> WindowResult<R>
    where
        T: FnOnce(Vec<WindowMonitor>) -> R,
    {
        let mut monitors = glfw_request!(
            RequestConnectedMonitors,
            ConnectedMonitors(monitors),
            monitors
        )?;
        let mut wrappers = Vec::new();
        for monitor in monitors.iter_mut() {
            wrappers.push(WindowMonitor { monitor });
        }
        Ok(f(wrappers))
    }

    /// Allows you to perform operations on the primary monitor,
    /// if it is present
    ///
    /// # Arguments
    /// * `f` - The function to receive the monitor option
    ///
    /// # Returns
    /// * Whatever `f` returns
    pub fn with_primary_monitor<T, R>(f: T) -> WindowResult<R>
    where
        T: FnOnce(Option<WindowMonitor>) -> R,
    {
        let mut monitor = glfw_request!(
            RequestPrimaryMonitor,
            PrimaryMonitor(monitor),
            monitor
        )?;
        let monitor = match monitor {
            Some(ref mut monitor) => Some(WindowMonitor { monitor }),
            None => None
        };
        Ok(f(monitor))
    }

    /// Returns a `WindowVideoMode` struct of the current video mode
    /// in use by this monitor
    ///
    /// # Returns
    /// * The current video mode, if it is present
    pub fn get_current_video_mode(&self) -> Option<VideoMode> {
        let mode = glfw_request!(
            RequestCurrentVideoMode(self.monitor.custom_clone()),
            CurrentVideoMode(mode),
            mode
        ).unwrap();

        match mode {
            Some(mode) => Some(VideoMode { mode }),
            None => None,
        }
    }

    /// Retrieves a list of all available video modes for this monitor
    ///
    /// # Returns
    /// * A list of all the video modes as `WindowVideoMode` structs
    pub fn get_video_modes(&self) -> Vec<VideoMode> {
        let raw = glfw_request!(
            RequestVideoModes(self.monitor.custom_clone()),
            VideoModes(modes),
            modes
        ).unwrap();

        let mut modes = Vec::with_capacity(raw.len());
        for mode in raw.into_iter() {
            modes.push(VideoMode { mode });
        }
        modes
    }

    /// # Returns
    /// * The position of this monitor in the virtual workspace
    pub fn get_virtual_pos(&self) -> (i32, i32) {
        glfw_request!(
            RequestVirtualPos(self.monitor.custom_clone()),
            VirtualPos(x, y),
            (x, y)
        ).unwrap()
    }

    /// # Returns
    /// * The physical size of this monitor
    pub fn get_physical_size(&self) -> (i32, i32) {
        glfw_request!(
            RequestPhysicalSize(self.monitor.custom_clone()),
            PhysicalSize(width, height),
            (width, height)
        ).unwrap()
    }

    /// # Returns
    /// * The content scale of this monitor
    pub fn get_content_scale(&self) -> (f32, f32) {
        glfw_request!(
            RequestContentScale(self.monitor.custom_clone()),
            ContentScale(scalex, scaley),
            (scalex, scaley)
        ).unwrap()
    }

    /// Retrieves the work area of this monitor, meaning the region that is
    /// not occupied by any menu bars or other ui elements
    ///
    /// # Returns
    /// * The coordinates of the upmost and leftmost corner
    /// * The width and height of the quad
    pub fn get_work_area(&self) -> (i32, i32, i32, i32) {
        glfw_request!(
            RequestWorkArea(self.monitor.custom_clone()),
            WorkArea(x, y, width, height),
            (x, y, width, height)
        ).unwrap()
    }

    /// # Returns
    /// * A human readable name for this monitor
    pub fn get_name(&self) -> Option<String> {
        glfw_request!(
            RequestName(self.monitor.custom_clone()),
            Name(name),
            name
        ).unwrap()
    }

    /// Allows you to set the gamma of this monitor
    ///
    /// # Arguments
    /// * `gamma` - The new gamma
    pub fn set_gamma(&mut self, gamma: f32) {
        glfw_request!(
            Gamma(self.monitor.custom_clone(), gamma),
            GOk,
            ()
        ).unwrap();
    }
}
