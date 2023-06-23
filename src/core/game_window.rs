use crate::core::entry_point;
use crate::core::entry_point::register_task;
use crate::log;
use crate::settings::GameSettings;
use crate::settings::Setting::Str;
use crate::utils::types::Vector2i;
use crate::utils::{file_util, Logger};
use glfw::ffi::glfwSetErrorCallback;
use glfw::{
    Callback, Context, Cursor, Error, ErrorCallback, Glfw, Monitor, SwapInterval, VidMode, Window,
    WindowEvent, WindowHint, WindowMode,
};
use image::RgbaImage;
use std::cell::RefCell;
use std::cmp::min;
use std::fmt::{format, Display, Formatter};
use std::ops::DerefMut;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{io, mem};

static GLFW_INITIALIZED: Mutex<bool> = Mutex::new(false);

struct GlfwContainer {
    glfw: Option<Glfw>,
    warned: bool,
}

impl GlfwContainer {
    fn new_failure() -> Self {
        Self {
            glfw: None,
            warned: true,
        }
    }

    fn new_success(glfw: Glfw) -> Self {
        Self {
            glfw: Some(glfw),
            warned: false,
        }
    }

    fn was_warned(&mut self) -> bool {
        mem::replace(&mut self.warned, true)
    }
}

thread_local! {
    static GLFW: RefCell<GlfwContainer> = {
        // Check if already initialized
        let mut value = GLFW_INITIALIZED.lock().unwrap();
        let was_initialized = mem::replace(&mut *value, true);
        if was_initialized {
            log!(Warning, "Tried to initialize GLFW but it was already initialized in this process before");
            return RefCell::new(GlfwContainer::new_failure());
        }

        entry_point::register_task(|| {
            with_glfw(|glfw| {
                glfw.poll_events();
            });
        });

        // Initialize glfw
        RefCell::new(match glfw::init(LOG_ERRORS) {
            Ok(glfw) => {
                GlfwContainer::new_success(glfw)
            },
            Err(error) => {
                log!(Fatal, &error, "Error initializing GLFW");
                GlfwContainer::new_failure()
            }
        })
    };
}

fn with_glfw<T, R>(f: T) -> Result<R, WindowError>
where
    T: FnOnce(&mut Glfw) -> R,
{
    GLFW.with(|cell| match cell.try_borrow_mut() {
        Ok(mut container) => match container.deref_mut().glfw {
            Some(ref mut glfw) => Ok(f(glfw)),
            None => {
                if !container.was_warned() {
                    log!(
                        Fatal,
                        "No GLFW instance in this thread. This could either mean that this is not the main thread, or that GLFW initialization failed"
                    );
                }
                Err(WindowError::InternalError)
            }
        },
        Err(error) => {
            log!(
                Fatal,
                &error,
                "Couldnt get a mutable reference to the glfw instance"
            );
            Err(WindowError::InternalError)
        }
    })
}

fn log_errors(error: Error, description: String, _: &()) {
    log!(Error, &error, "{}", description);
}

static LOG_ERRORS: Option<ErrorCallback<()>> = Some(Callback {
    f: log_errors as fn(Error, String, &()),
    data: (),
});

pub type WindowResult<T> = Result<T, WindowError>;

#[derive(Debug)]
pub enum WindowError {
    InternalError,
    CreationError,
    IOError(io::Error),
    CursorError,
}

impl std::error::Error for WindowError {}

impl Display for WindowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::InternalError => write!(f, "InternalError"),
            WindowError::CreationError => write!(f, "CreationError"),
            WindowError::IOError(error) => write!(f, "IOError: {}", error),
            WindowError::CursorError => write!(f, "CursorError"),
        }
    }
}

/// All the possible vsync settings
///
/// # Meaning
///
/// * `Disabled` - No vsync
/// * `Enabled` - Vsync is enabled
/// * `Adaptive` - Let the system decide
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WindowVsyncMode {
    Disabled,
    Enabled,
    Adaptive,
}

/// Represents a window monitor and provides
/// some utility functions
pub struct WindowMonitor<'a> {
    monitor: &'a mut Monitor,
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
    pub fn with_monitors<T, R>(mut f: T) -> WindowResult<R>
    where
        T: FnMut(Vec<WindowMonitor>) -> R,
    {
        with_glfw(|glfw| {
            glfw.with_connected_monitors(|glfw, monitors| {
                let mut wrappers = Vec::new();
                for monitor in monitors.iter_mut() {
                    wrappers.push(WindowMonitor { monitor });
                }
                f(wrappers)
            })
        })
    }

    /// Allows you to perform operations on the primary monitor,
    /// if it is present
    ///
    /// # Arguments
    /// * `f` - The function to receive the monitor option
    ///
    /// # Returns
    /// * Whatever `f` returns
    pub fn with_primary_monitor<T, R>(mut f: T) -> WindowResult<R>
    where
        T: FnMut(Option<WindowMonitor>) -> R,
    {
        with_glfw(|glfw| {
            glfw.with_primary_monitor(|glfw, monitor| match monitor {
                Some(monitor) => f(Some(WindowMonitor { monitor })),
                None => f(None),
            })
        })
    }

    /// Returns a `WindowVideoMode` struct of the current video mode
    /// in use by this monitor
    ///
    /// # Returns
    /// * The current video mode, if it is present
    pub fn get_current_video_mode(&self) -> Option<WindowVideoMode> {
        match self.monitor.get_video_mode() {
            Some(mode) => Some(WindowVideoMode { mode }),
            None => None,
        }
    }

    /// Retrieves a list of all available video modes for this monitor
    ///
    /// # Returns
    /// * A list of all the video modes as `WindowVideoMode` structs
    pub fn get_video_modes(&self) -> Vec<WindowVideoMode> {
        let raw = self.monitor.get_video_modes();
        let mut modes = Vec::with_capacity(raw.len());
        for mode in raw.into_iter() {
            modes.push(WindowVideoMode { mode });
        }
        modes
    }

    /// # Returns
    /// * The position of this monitor in the virtual workspace
    pub fn get_virtual_pos(&self) -> (i32, i32) {
        self.monitor.get_pos()
    }

    /// # Returns
    /// * The physical size of this monitor
    pub fn get_physical_size(&self) -> (i32, i32) {
        self.monitor.get_physical_size()
    }

    /// # Returns
    /// * The content scale of this monitor
    pub fn get_content_scale(&self) -> (f32, f32) {
        self.monitor.get_content_scale()
    }

    /// Retrieves the work area of this monitor, meaning the region that is
    /// not occupied by any menu bars or other ui elements
    ///
    /// # Returns
    /// * The coordinates of the upmost and leftmost corner
    /// * The width and height of the quad
    pub fn get_work_area(&self) -> (i32, i32, i32, i32) {
        self.monitor.get_workarea()
    }

    /// # Returns
    /// * A human readable name for this monitor
    pub fn get_name(&self) -> Option<String> {
        self.monitor.get_name()
    }

    /// Allows you to set the gamma of this monitor
    ///
    /// # Arguments
    /// * `gamma` - The new gamma
    pub fn set_gamma(&mut self, gamma: f32) {
        self.monitor.set_gamma(gamma);
    }
}

/// Represents a possible video mode for a monitor
pub struct WindowVideoMode {
    mode: VidMode,
}

impl WindowVideoMode {
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
}

#[derive(Debug)]
/// Represents a window with graphics capabilities
pub struct GameWindow {
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,
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
    pub fn new(settings: &Arc<GameSettings>) -> WindowResult<GameWindow> {
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

        let (window, receiver) = with_glfw(|glfw| {
            // Set window hints
            glfw.default_window_hints();
            glfw.window_hint(WindowHint::Resizable(is_resizable));
            glfw.window_hint(WindowHint::Maximized(is_maximized));
            glfw.window_hint(WindowHint::Visible(false));
            glfw.window_hint(WindowHint::OpenGlDebugContext(is_debug_context));
            glfw.window_hint(WindowHint::TransparentFramebuffer(is_transparent));

            // Create glfw window
            return match glfw.create_window(
                windowed_size.x as u32,
                windowed_size.y as u32,
                title,
                WindowMode::Windowed,
            ) {
                Some(window) => Ok((window.0, window.1)),
                None => Err(WindowError::CreationError),
            };
        })??;

        let saved_pos = window.get_pos();

        // Construct game window object
        let mut game_window = Self {
            window,
            receiver,
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
                game_window.set_fullscreen_index(
                    monitor_index as u64,
                    (fullscreen_size.x, fullscreen_size.y),
                );
            } else {
                game_window.set_fullscreen_primary((fullscreen_size.x, fullscreen_size.y));
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
            game_window.set_icon_path(paths).unwrap_or(());
        }

        Ok(game_window)
    }

    /// # Returns
    /// * Whether or not the window is resizable
    pub fn is_resizable(&self) -> bool {
        self.window.is_resizable()
    }

    /// Change whether or not the window is resizable
    ///
    /// # Arguments
    /// * `resizable` - The new value
    pub fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    /// # Returns
    /// * The size of the window in screen coordinates (not pixels!)
    pub fn get_size(&self) -> (i32, i32) {
        self.window.get_size()
    }

    /// Change the size of the window.
    /// An attempt is made to satisfy the given size, but it
    /// will be made to fit given the size limits of the window.
    ///
    /// # Arguments
    /// * `size` - The new size in screen coordinates (not pixels!)
    pub fn set_size(&mut self, size: (i32, i32)) {
        self.window.set_size(
            if size.0 > 0 { size.0 } else { 1 },
            if size.1 > 0 { size.1 } else { 1 },
        );
    }

    /// # Returns
    /// * Whether or not the window is visible
    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    /// Changes the visibility of the window
    ///
    /// # Arguments
    /// * `visible` - The new visibility
    pub fn set_visible(&mut self, visible: bool) {
        if visible {
            self.window.show();
        } else {
            self.window.hide();
        }
    }

    /// # Returns
    /// * Whether or not the close flag is true
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    /// Changes the close flag
    ///
    /// # Arguments
    /// * `should_close` - The new value
    pub fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
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
        self.title = title;
        self.window.set_title(&self.title);
    }

    /// # Returns
    /// * The position of the window in screen coordinates (not pixels!)
    pub fn get_position(&self) -> (i32, i32) {
        self.window.get_pos()
    }

    /// Changes the position of the window
    ///
    /// # Arguments
    /// * `position` - The new position in screen coordinates (not pixels!)
    pub fn set_position(&mut self, position: (i32, i32)) {
        self.window.set_pos(position.0, position.1);
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
    pub fn set_size_limits(&mut self, size_limits: (i32, i32, i32, i32)) {
        self.window.set_size_limits(
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
            },
        );

        // Validate size limits
        let min_valid = size_limits.0 > 0 && size_limits.1 > 0;
        let max_valid = size_limits.2 > 0 && size_limits.3 > 0;

        self.size_limits = (
            if min_valid { size_limits.0 } else { -1 },
            if min_valid { size_limits.1 } else { -1 },
            if max_valid { size_limits.2 } else { -1 },
            if max_valid { size_limits.3 } else { -1 },
        );
    }

    /// Swap the back and front buffers of the window
    pub fn swap(&mut self) {
        self.window.swap_buffers();
    }

    /// Changes the vsync mode
    ///
    /// # Arguments
    /// * `vsync` - The new vsync mode
    pub fn set_vsync(&mut self, vsync: WindowVsyncMode) -> WindowResult<()> {
        let mode = match vsync {
            WindowVsyncMode::Disabled => SwapInterval::None,
            WindowVsyncMode::Enabled => SwapInterval::Sync(1),
            WindowVsyncMode::Adaptive => SwapInterval::Adaptive,
        };
        with_glfw(|glfw| glfw.set_swap_interval(mode))
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
    pub fn set_fullscreen_index(&mut self, monitor_index: u64, preferred_res: (i32, i32)) {
        WindowMonitor::with_monitors(|monitors| {
            let monitor;
            if let Some(monitor_) = monitors.get(monitor_index as usize) {
                monitor = monitor_;
            } else if let Some(monitor_) = monitors.get(0) {
                monitor = monitor_;
            } else {
                self.set_fullscreen_primary(preferred_res);
                return;
            }
            self.set_fullscreen_monitor(monitor, preferred_res);
        });
    }

    /// Sets the window in fullscreen mode on the primary monitor.
    /// An attempt is made to retrieve the primary monitor, but if is not found
    /// this function returns without doing anything.
    ///
    /// # Arguments
    /// * `preferred_res` - The preferred resolution
    pub fn set_fullscreen_primary(&mut self, preferred_res: (i32, i32)) {
        WindowMonitor::with_primary_monitor(|primary| {
            if let Some(primary) = primary {
                self.set_fullscreen_monitor(&primary, preferred_res)
            }
        });
    }

    /// Sets the window in fullscreen mode on the given monitor.
    ///
    /// # Arguments
    /// * `monitor` - The monitor
    /// * `preferred_res` - The preferred resolution
    pub fn set_fullscreen_monitor(&mut self, monitor: &WindowMonitor, preferred_res: (i32, i32)) {
        self.saved_size = self.window.get_size();
        self.saved_pos = self.window.get_pos();

        self.window.set_monitor(
            WindowMode::FullScreen(monitor.monitor),
            0,
            0,
            preferred_res.0 as u32,
            preferred_res.1 as u32,
            None,
        );
        self.fullscreen = true;
    }

    /// Sets the window in windowed mode if it is not already so
    pub fn set_windowed(&mut self) {
        if !self.fullscreen {
            return;
        }

        // Restore the saved size
        self.window.set_monitor(
            WindowMode::Windowed,
            self.saved_pos.0,
            self.saved_pos.1,
            self.saved_size.0 as u32,
            self.saved_size.1 as u32,
            None,
        );
        self.fullscreen = false;
    }

    /// Changes the icon of the window
    ///
    /// # Arguments
    /// * `images` - A list of different resolution image representations of the icon
    pub fn set_icon(&mut self, images: Vec<RgbaImage>) {
        self.window.set_icon(images);
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

        let cursor = Cursor::create(cursor, center.0, center.1);
        self.window.set_cursor(Some(cursor));
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

    /// Resets the cursor to the system default
    pub fn reset_cursor(&mut self) {
        self.window.set_cursor(None);
    }

    /// Makes the context of the window current for the calling thread
    pub fn make_context_current(&mut self) {
        self.window.make_current();
    }

    /// # Returns
    /// * Whether the window is focused or not
    pub fn is_focused(&self) -> bool {
        self.window.is_focused()
    }

    /// Request focus on the window
    pub fn request_focus(&mut self) {
        self.window.focus();
    }

    /// # Returns
    /// * The opacity of the window
    pub fn get_opacity(&self) -> f32 {
        self.window.get_opacity()
    }

    /// Changes the opacity of the window
    ///
    /// # Arguments
    /// * `opacity` - The new opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.window.set_opacity(opacity);
    }

    /// # Returns
    /// * Whether or not the window is iconified
    pub fn is_iconified(&self) -> bool {
        self.window.is_iconified()
    }

    /// Reduces the window to an icon or restores it
    ///
    /// # Arguments
    /// * `iconified` - Whether or not the window should be iconified after this call
    pub fn set_iconified(&mut self, iconified: bool) {
        if iconified {
            self.window.iconify();
        } else {
            self.window.restore();
        }
    }

    /// # Returns
    /// * Whether or not the window is maximized
    pub fn is_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    /// Maximizes the window or restores it
    ///
    /// # Arguments
    /// * `maximized` - Whether or not the window should be iconified after this call
    pub fn set_maximized(&mut self, maximized: bool) {
        if maximized {
            self.window.maximize();
        } else {
            self.window.restore();
        }
    }

    /// # Returns
    /// * Whether or not this window is decorated (has an action bar / borders)
    pub fn is_decorated(&self) -> bool {
        self.window.is_decorated()
    }

    /// Changes the decorated flag
    ///
    /// # Arguments
    /// * `decorated` - Whether or not the window should be decorated
    pub fn set_decorated(&mut self, decorated: bool) {
        self.window.set_decorated(decorated);
    }

    /// # Returns
    /// * Whether or not the cursor is currently hovering the window
    pub fn is_hovered(&self) -> bool {
        self.window.is_hovered()
    }

    /// # Returns
    /// * Whether or not the window was created with a framebuffer that allows transparency
    pub fn is_transparent(&self) -> bool {
        self.window.is_framebuffer_transparent()
    }

    /// # Returns
    /// * Whether or not the window was created with a debug context
    pub fn is_debug_context(&self) -> bool {
        self.window.is_opengl_debug_context()
    }

    /// # Returns
    /// * The size in pixels of the underlying framebuffer
    pub fn get_framebuffer_size(&self) -> (i32, i32) {
        self.window.get_framebuffer_size()
    }

    /// Updates all windows
    pub fn poll_events() {
        with_glfw(|glfw| {
            glfw.poll_events();
        });
    }
}
