use crate::input::InputDispatcher;
use crate::log;
use crate::settings::GameSettings;
use crate::settings::Setting::Str;
use crate::utils::file_util;
use crate::utils::request_pipe::RequestPipe;
use crate::utils::types::Vector2i;
use crate::window::packets::{GlfwPacket, WindowPacket};
use crate::window::window_manager::*;
use crate::window::{cursor_mode, window_manager, VsyncMode, WindowMonitor};
use glfw::{Context, RenderContext, SwapInterval, WindowHint};
use image::RgbaImage;
use std::path::Path;
use std::sync;
use std::sync::Arc;

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

macro_rules! window_request {
    ($pipe:expr, $packet:ident ( $($request_args:expr),+ ), $response:ident ( $($response_args:ident),+ ), $ret:expr) => {{
        match $pipe.request(WindowPacket::$packet( $($request_args),* )) {
            WindowPacket::$response ( $($response_args),* ) => {Ok($ret)},
            WindowPacket::WError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($pipe:expr, $packet:ident ( $($request_args:expr),+ ), $response:ident, $ret:expr) => {{
        match $pipe.request(WindowPacket::$packet( $($request_args),* )) {
            WindowPacket::$response => {Ok($ret)},
            WindowPacket::WError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($pipe:expr, $packet:ident, $response:ident ( $($response_args:ident),+ ), $ret:expr) => {{
        match $pipe.request(WindowPacket::$packet) {
            WindowPacket::$response ( $($response_args),* ) => {Ok($ret)},
            WindowPacket::WError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
    ($pipe:expr, $packet:ident, $response:ident, $ret:expr) => {{
        match $pipe.request(WindowPacket::$packet) {
            WindowPacket::$response => {Ok($ret)},
            WindowPacket::WError(error) => {Err(error)}
            _ => {Err(WindowError::InternalError)}
        }
    }};
}

/// Represents a window with graphics capabilities
pub struct GameWindow {
    window_pipe: Arc<RequestPipe<WindowPacket>>,
    render_context: RenderContext,
    title: String,
    size_limits: (i32, i32, i32, i32),
    fullscreen: bool,
    saved_size: (i32, i32),
    saved_pos: (i32, i32),
}

impl GameWindow {
    /// Creates a new window using the given settings
    ///
    /// # Returns
    /// * If no errors occurs, the newly created window is returned
    pub fn new(settings: &GameSettings) -> WindowResult<GameWindow> {
        // Try to init glfw
        window_manager::maybe_init_glfw()?;

        // Get window settings
        let empty_string = "".to_string();

        let windowed_size = settings
            .get("window.size")
            .as_int_vec2_or(Vector2i::new(256, 256));

        let min_size = settings
            .get("window.minimumSize")
            .as_int_vec2_or(Vector2i::new(64, 64));

        let max_size = settings
            .get("window.maximumSize")
            .as_int_vec2_or(Vector2i::new(-1, -1)); // Meaning no max size

        let is_fullscreen = settings.get("window.fullscreen").as_boolean_or(false);

        let is_resizable = settings.get("window.resizable").as_boolean_or(true);

        let is_maximized = settings.get("window.maximized").as_boolean_or(true);

        let is_transparent = settings.get("window.transparent").as_boolean_or(false);

        let is_debug_context = settings.get("window.debugContext").as_boolean_or(false);

        let monitor_index = settings
            .get("window.fullscreenMonitor")
            .as_signed_int_or(-1);

        let title = settings.get("window.title");
        let title = title.as_string_or(&empty_string);

        // Get fullscreen info
        let fullscreen_size = settings
            .get("window.fullscreenResolution")
            .as_int_vec2_or(Vector2i::new(1920, 1080));

        glfw_request!(SetDefaultHints, GOk, ())?;
        glfw_request!(SetWindowHint(WindowHint::Resizable(is_resizable)), GOk, ())?;
        glfw_request!(SetWindowHint(WindowHint::Maximized(is_maximized)), GOk, ())?;
        glfw_request!(SetWindowHint(WindowHint::Visible(false)), GOk, ())?;
        glfw_request!(
            SetWindowHint(WindowHint::OpenGlDebugContext(is_debug_context)),
            GOk,
            ()
        )?;
        glfw_request!(
            SetWindowHint(WindowHint::TransparentFramebuffer(is_transparent)),
            GOk,
            ()
        )?;
        let (window_pipe, render_context) = glfw_request!(
            CreateWindow(
                windowed_size.x as u32,
                windowed_size.y as u32,
                title.clone()
            ),
            WindowCreated(window_pipe, render_context),
            (window_pipe, render_context)
        )?;

        let saved_pos = window_request!(window_pipe, RequestPosition, Position(x, y), (x, y))?;

        // Construct game window object
        let mut game_window = Self {
            window_pipe,
            render_context,
            title: title.clone(),
            size_limits: (0, 0, 0, 0),
            fullscreen: false,
            saved_size: (windowed_size.x, windowed_size.y),
            saved_pos,
        };

        // Apply some last settings
        game_window.set_size_limits((min_size.x, min_size.y, max_size.x, max_size.y));

        // Fix the fullscreen monitor used
        if is_fullscreen {
            if monitor_index >= 0 {
                let _ = game_window.set_fullscreen_index(
                    monitor_index as u64,
                    (fullscreen_size.x, fullscreen_size.y),
                );
            } else {
                let _ = game_window.set_fullscreen_primary((fullscreen_size.x, fullscreen_size.y));
            }
        }

        // Try to apply an icon
        let mut paths: Vec<&Path> = Vec::new();
        let icon16;
        let icon32;

        if let Str(path) = settings.get("window.icon16") {
            icon16 = path;
            paths.push(Path::new(&icon16));
        }
        if let Str(path) = settings.get("window.icon32") {
            icon32 = path;
            paths.push(Path::new(&icon32));
        }

        if paths.len() > 0 {
            let _ = game_window.set_icon_path(paths);
        }

        Ok(game_window)
    }

    /// # Returns
    /// * Whether or not the window is resizable
    pub fn is_resizable(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestResizable,
            Resizable(resizable),
            resizable
        )
        .unwrap()
    }

    /// Change whether or not the window is resizable
    ///
    /// # Arguments
    /// * `resizable` - The new value
    pub fn set_resizable(&mut self, resizable: bool) {
        window_request!(self.window_pipe, Resizable(resizable), WOk, ()).unwrap()
    }

    /// # Returns
    /// * The size of the window in screen coordinates (not pixels!)
    pub fn get_size(&self) -> (i32, i32) {
        window_request!(
            self.window_pipe,
            RequestSize,
            Size(width, height),
            (width, height)
        )
        .unwrap()
    }

    /// Change the size of the window.
    /// An attempt is made to satisfy the given size, but it
    /// will be made to fit given the size limits of the window.
    ///
    /// # Arguments
    /// * `size` - The new size in screen coordinates (not pixels!)
    pub fn set_size(&mut self, size: (i32, i32)) {
        window_request!(self.window_pipe, Size(size.0, size.1), WOk, ()).unwrap()
    }

    /// # Returns
    /// * Whether or not the window is visible
    pub fn is_visible(&self) -> bool {
        window_request!(self.window_pipe, RequestVisible, Visible(visible), visible).unwrap()
    }

    /// Changes the visibility of the window
    ///
    /// # Arguments
    /// * `visible` - The new visibility
    pub fn set_visible(&mut self, visible: bool) {
        window_request!(self.window_pipe, Visible(visible), WOk, ()).unwrap()
    }

    /// # Returns
    /// * Whether or not the close flag is true
    pub fn should_close(&self) -> bool {
        self.render_context.should_close()
    }

    /// Changes the close flag
    ///
    /// # Arguments
    /// * `should_close` - The new value
    pub fn set_should_close(&mut self, should_close: bool) {
        self.render_context.set_should_close(should_close);
    }

    /// # Returns
    /// * The title of the window
    pub fn get_title(&self) -> &String {
        &self.title
    }

    /// Changes the title of the window
    ///
    /// # Arguments
    /// * `title` - The new title
    pub fn set_title(&mut self, title: String) {
        window_request!(self.window_pipe, Title(title), WOk, ()).unwrap()
    }

    /// # Returns
    /// * The position of the window in screen coordinates (not pixels!)
    pub fn get_position(&self) -> (i32, i32) {
        window_request!(self.window_pipe, RequestPosition, Position(x, y), (x, y)).unwrap()
    }

    /// Changes the position of the window
    ///
    /// # Arguments
    /// * `position` - The new position in screen coordinates (not pixels!)
    pub fn set_position(&mut self, position: (i32, i32)) {
        window_request!(self.window_pipe, Position(position.0, position.1), WOk, ()).unwrap()
    }

    /// # Returns
    /// * The last known size limits of the window in screen coordinates (not pixels!)
    pub fn get_size_limits(&self) -> (i32, i32, i32, i32) {
        self.size_limits
    }

    /// Changes the size limits of the window
    ///
    /// # Arguments
    /// * `size_limits` - The new size limits in screen coordinates (not pixels!)
    pub fn set_size_limits(&mut self, mut size_limits: (i32, i32, i32, i32)) {
        // Validate size limits
        let min_valid = size_limits.0 > 0 && size_limits.1 > 0;
        let max_valid = size_limits.2 > 0 && size_limits.3 > 0;

        size_limits = (
            if min_valid { size_limits.0 } else { -1 },
            if min_valid { size_limits.1 } else { -1 },
            if max_valid { size_limits.2 } else { -1 },
            if max_valid { size_limits.3 } else { -1 },
        );

        self.size_limits = size_limits;

        window_request!(
            self.window_pipe,
            SizeLimits(
                if size_limits.0 > 0 {
                    Some(size_limits.0 as u32)
                } else {
                    None
                },
                if size_limits.1 > 0 {
                    Some(size_limits.1 as u32)
                } else {
                    None
                },
                if size_limits.2 > 0 {
                    Some(size_limits.2 as u32)
                } else {
                    None
                },
                if size_limits.3 > 0 {
                    Some(size_limits.3 as u32)
                } else {
                    None
                }
            ),
            WOk,
            ()
        )
        .unwrap()
    }

    /// Swap the back and front buffers of the window
    pub fn swap(&mut self) {
        self.render_context.swap_buffers();
    }

    /// Changes the vsync mode
    ///
    /// # Arguments
    /// * `vsync` - The new vsync mode
    pub fn set_vsync(&mut self, vsync: VsyncMode) {
        match vsync {
            VsyncMode::Disabled => glfw_request!(SetSwapInterval(SwapInterval::None), GOk, ()),
            VsyncMode::Enabled => glfw_request!(SetSwapInterval(SwapInterval::Sync(1)), GOk, ()),
            VsyncMode::Adaptive => glfw_request!(SetSwapInterval(SwapInterval::Adaptive), GOk, ()),
        }
        .unwrap();
    }

    /// # Returns
    /// * Whether the window is fullscreen or not
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Sets the window in fullscreen mode on the monitor with the given index.
    /// An attempt will be made to find the specified monitor, but if it is not found
    /// this function falls back to `set_fullscreen_primary`
    /// Please prefer iterating over monitors manually rather than using this function
    ///
    /// # Arguments
    /// * `monitor_index` - The index of the monitor
    /// * `preferred_res` - The preferred resolution
    pub fn set_fullscreen_index(
        &mut self,
        monitor_index: u64,
        preferred_res: (i32, i32),
    ) -> WindowResult<()> {
        WindowMonitor::with_monitors(|monitors| {
            let monitor;
            if let Some(monitor_) = monitors.get(monitor_index as usize) {
                monitor = monitor_;
            } else if let Some(monitor_) = monitors.get(0) {
                monitor = monitor_;
            } else {
                return self.set_fullscreen_primary(preferred_res);
            }
            self.set_fullscreen_monitor(monitor, preferred_res)
        })?
    }

    /// Sets the window in fullscreen mode on the primary monitor.
    /// An attempt is made to retrieve the primary monitor, but if is not found
    /// this function returns without doing anything.
    ///
    /// # Arguments
    /// * `preferred_res` - The preferred resolution
    pub fn set_fullscreen_primary(&mut self, preferred_res: (i32, i32)) -> WindowResult<()> {
        WindowMonitor::with_primary_monitor(|primary| {
            if let Some(primary) = primary {
                self.set_fullscreen_monitor(&primary, preferred_res)
            } else {
                Err(WindowError::NoMonitor)
            }
        })?
    }

    /// Sets the window in fullscreen mode on the given monitor.
    ///
    /// # Arguments
    /// * `monitor` - The monitor
    /// * `preferred_res` - The preferred resolution
    pub fn set_fullscreen_monitor(
        &mut self,
        monitor: &WindowMonitor,
        preferred_res: (i32, i32),
    ) -> WindowResult<()> {
        self.saved_size = self.get_size();
        self.saved_pos = self.get_position();

        window_request!(
            self.window_pipe,
            SetMonitor(
                Some(monitor.monitor.custom_clone()),
                0,
                0,
                preferred_res.0 as u32,
                preferred_res.1 as u32,
                None
            ),
            WOk,
            ()
        )
        .unwrap();

        self.fullscreen = true;
        Ok(())
    }

    /// Sets the window in windowed mode if it is not already so
    pub fn set_windowed(&mut self) {
        if !self.fullscreen {
            return;
        }

        // Restore the saved size
        window_request!(
            self.window_pipe,
            SetMonitor(
                None,
                self.saved_pos.0,
                self.saved_pos.1,
                self.saved_size.0 as u32,
                self.saved_size.1 as u32,
                None
            ),
            WOk,
            ()
        )
        .unwrap();

        self.fullscreen = false;
    }

    /// Changes the icon of the window
    ///
    /// # Arguments
    /// * `images` - A list of different resolution image representations of the icon
    pub fn set_icon(&mut self, images: Vec<RgbaImage>) {
        window_request!(self.window_pipe, Icon(images), WOk, ()).unwrap();
    }

    /// Changes the icon of the window loading the images from the given paths
    ///
    /// # Arguments
    /// * `paths` - The paths where the images are located relative to the working dir
    pub fn set_icon_path(&mut self, paths: Vec<&Path>) -> WindowResult<()> {
        let mut images: Vec<RgbaImage> = Vec::new();
        for path in paths.iter() {
            match file_util::path_to_image(*path) {
                Ok(img) => {
                    images.push(img.to_rgba8());
                }
                Err(error) => {
                    log!(Error, &error, "Couldn't load a window icon");
                    return Err(WindowError::IOError(error));
                }
            }
        }
        self.set_icon(images);
        Ok(())
    }

    /// Changes the cursor of the window
    ///
    /// # Arguments
    /// * `cursor` - The image of the new cursor. Size must be equal or lower than 256x256
    /// * `center` - The coordinates of the center of the cursor relative to the image
    pub fn set_cursor(&mut self, cursor: RgbaImage, center: (u32, u32)) -> WindowResult<()> {
        // Cursor larger than this size create graphical glitches on X11
        let max_size = 256;
        if cursor.width() > max_size || cursor.height() > max_size {
            log!(Error, "Cursor size cannot be larger than 256x256");
            return Err(WindowError::CursorError);
        }

        let cursor = glfw_request!(
            CreateCursor(cursor, center.0, center.1),
            CursorCreated(cursor),
            cursor
        )?;
        window_request!(self.window_pipe, SetCursor(Some(cursor)), WOk, ()).unwrap();
        Ok(())
    }

    /// Changes the cursor of the window, loading the image from the given path
    ///
    /// # Arguments
    /// * `path` - The path where the image is located relative to the working dir
    /// * `center` - The coordinates of the center of the cursor relative to the image
    pub fn set_cursor_path(&mut self, path: &Path, center: (u32, u32)) -> WindowResult<()> {
        match file_util::path_to_image(path) {
            Ok(img) => self.set_cursor(img.to_rgba8(), center),
            Err(error) => {
                log!(Error, &error, "Couldn't load a cursor");
                return Err(WindowError::IOError(error));
            }
        }
    }

    /// Changes the cursor mode of the window.
    /// Use this function for example when pausing and resuming a game
    /// in order to capture or free the mouse cursor
    ///
    /// # Arguments
    /// * `mode` - The new cursor mode
    pub fn set_cursor_mode(&mut self, mode: cursor_mode::CursorMode) {
        let mode: glfw::CursorMode = match mode {
            cursor_mode::CursorMode::Captured => glfw::CursorMode::Disabled,
            cursor_mode::CursorMode::Invisible => glfw::CursorMode::Hidden,
            cursor_mode::CursorMode::Normal => glfw::CursorMode::Normal,
        };
        window_request!(self.window_pipe, SetCursorMode(mode), WOk, ()).unwrap();
    }

    /// Resets the cursor to the system default
    pub fn reset_cursor(&mut self) {
        window_request!(self.window_pipe, SetCursor(None), WOk, ()).unwrap();
    }

    /// Makes the context of the window current for the calling thread
    pub fn make_context_current(&mut self) {
        self.render_context.make_current();
    }

    /// # Returns
    /// * Whether the window is focused or not
    pub fn is_focused(&self) -> bool {
        window_request!(self.window_pipe, RequestFocused, Focused(focused), focused).unwrap()
    }

    /// Request focus on the window
    pub fn request_focus(&mut self) {
        window_request!(self.window_pipe, Focused(true), WOk, ()).unwrap();
    }

    /// # Returns
    /// * The opacity of the window
    pub fn get_opacity(&self) -> f32 {
        window_request!(self.window_pipe, RequestOpacity, Opacity(opacity), opacity).unwrap()
    }

    /// Changes the opacity of the window
    ///
    /// # Arguments
    /// * `opacity` - The new opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        window_request!(self.window_pipe, Opacity(opacity), WOk, ()).unwrap();
    }

    /// # Returns
    /// * Whether or not the window is iconified
    pub fn is_iconified(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestIconified,
            Iconified(iconified),
            iconified
        )
        .unwrap()
    }

    /// Reduces the window to an icon or restores it
    ///
    /// # Arguments
    /// * `iconified` - Whether or not the window should be iconified after this call
    pub fn set_iconified(&mut self, iconified: bool) {
        window_request!(self.window_pipe, Iconified(iconified), WOk, ()).unwrap();
    }

    /// # Returns
    /// * Whether or not the window is maximized
    pub fn is_maximized(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestMaximized,
            Maximized(maximized),
            maximized
        )
        .unwrap()
    }

    /// Maximizes the window or restores it
    ///
    /// # Arguments
    /// * `maximized` - Whether or not the window should be iconified after this call
    pub fn set_maximized(&mut self, maximized: bool) {
        window_request!(self.window_pipe, Maximized(maximized), WOk, ()).unwrap();
    }

    /// # Returns
    /// * Whether or not this window is decorated (has an action bar / borders)
    pub fn is_decorated(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestDecorated,
            Decorated(decorated),
            decorated
        )
        .unwrap()
    }

    /// Changes the decorated flag
    ///
    /// # Arguments
    /// * `decorated` - Whether or not the window should be decorated
    pub fn set_decorated(&mut self, decorated: bool) {
        window_request!(self.window_pipe, Decorated(decorated), WOk, ()).unwrap();
    }

    /// # Returns
    /// * Whether or not the cursor is currently hovering the window
    pub fn is_hovered(&self) -> bool {
        window_request!(self.window_pipe, RequestHovered, Hovered(hovered), hovered).unwrap()
    }

    /// # Returns
    /// * Whether or not the window was created with a framebuffer that allows transparency
    pub fn is_transparent(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestTransparent,
            Transparent(transparent),
            transparent
        )
        .unwrap()
    }

    /// # Returns
    /// * Whether or not the window was created with a debug context
    pub fn is_debug_context(&self) -> bool {
        window_request!(
            self.window_pipe,
            RequestDebugContext,
            DebugContext(debug_context),
            debug_context
        )
        .unwrap()
    }

    /// # Returns
    /// * The size in pixels of the underlying framebuffer
    pub fn get_framebuffer_size(&self) -> (i32, i32) {
        window_request!(
            self.window_pipe,
            RequestFrameBufferSize,
            FrameBufferSize(width, height),
            (width, height)
        )
        .unwrap()
    }

    /// Registers this window as an input device, sending its input events to the
    /// provided `InputDispatcher`.
    /// Calling this function repeatedly with a different `InputDispatcher` will
    /// override the previous.
    pub fn register_input_device(&mut self, dispatcher: sync::Weak<InputDispatcher>) {
        register_input(self.window_pipe.as_ref(), dispatcher);
    }

    /// Reverts the effects of [`GameWindow::register_input_device`]
    pub fn unregister_input_device(&mut self) {
        unregister_input(self.window_pipe.as_ref());
    }
}

impl Drop for GameWindow {
    fn drop(&mut self) {
        self.unregister_input_device();
        glfw_request!(DestroyWindow(self.window_pipe.clone()), GOk, ()).unwrap();
    }
}

unsafe impl Send for GameWindow {}
unsafe impl Sync for GameWindow {}
