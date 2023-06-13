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

thread_local! {
    static GLFW: RefCell<Option<Glfw>> = {
        // Check if already initialized
        let mut value = GLFW_INITIALIZED.lock().unwrap();
        let was_initialized = mem::replace(&mut *value, true);
        if was_initialized {
            log!(Warning, "Tried to initialize GLFW but it was already initialized in this process before");
            return RefCell::new(None);
        }

        // Initialize glfw
        RefCell::new(match glfw::init(LOG_ERRORS) {
            Ok(glfw) => {
                Some(glfw)
            },
            Err(error) => {
                log!(Fatal, &error, "Error initializing GLFW");
                None
            }
        })
    };
}

fn with_glfw<T, R>(f: T) -> Result<R, WindowError>
where
    T: FnOnce(&mut Glfw) -> R,
{
    GLFW.with(|cell| {
        let mut option = cell.borrow_mut();
        let glfw;
        if let Some(glfw_) = option.deref_mut() {
            glfw = glfw_;
        } else {
            log!(Fatal, "No GLFW instance in this thread. This could either mean that this is not the main thread, or that GLFW initialization failed");
            return Err(WindowError::InternalError);
        }

        Ok(f(glfw))
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

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WindowVsyncMode {
    Disabled,
    Enabled,
    Adaptive,
}

pub struct WindowMonitor<'a> {
    monitor: &'a mut Monitor,
}

impl<'a> WindowMonitor<'a> {
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

    pub fn get_current_video_mode(&self) -> Option<WindowVideoMode> {
        match self.monitor.get_video_mode() {
            Some(mode) => Some(WindowVideoMode { mode }),
            None => None,
        }
    }

    pub fn get_video_modes(&self) -> Vec<WindowVideoMode> {
        let raw = self.monitor.get_video_modes();
        let mut modes = Vec::with_capacity(raw.len());
        for mode in raw.into_iter() {
            modes.push(WindowVideoMode { mode });
        }
        modes
    }

    pub fn get_virtual_pos(&self) -> (i32, i32) {
        self.monitor.get_pos()
    }

    pub fn get_physical_size(&self) -> (i32, i32) {
        self.monitor.get_physical_size()
    }

    pub fn get_content_scale(&self) -> (f32, f32) {
        self.monitor.get_content_scale()
    }

    pub fn get_work_area(&self) -> (i32, i32, i32, i32) {
        self.monitor.get_workarea()
    }

    pub fn get_name(&self) -> Option<String> {
        self.monitor.get_name()
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.monitor.set_gamma(gamma);
    }
}

pub struct WindowVideoMode {
    mode: VidMode,
}

impl WindowVideoMode {
    pub fn get_size(&self) -> (u32, u32) {
        (self.mode.width, self.mode.height)
    }

    pub fn get_refresh_rate(&self) -> u32 {
        self.mode.refresh_rate
    }
}

#[derive(Debug)]
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

    pub fn is_resizable(&self) -> bool {
        self.window.is_resizable()
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    pub fn get_size(&self) -> (i32, i32) {
        self.window.get_size()
    }

    pub fn set_size(&mut self, size: (i32, i32)) {
        self.window.set_size(
            if size.0 > 0 { size.0 } else { 1 },
            if size.1 > 0 { size.1 } else { 1 },
        );
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    pub fn set_visible(&mut self, visible: bool) {
        if visible {
            self.window.show();
        } else {
            self.window.hide();
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.window.set_title(&self.title);
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.window.get_pos()
    }

    pub fn set_position(&mut self, position: (i32, i32)) {
        self.window.set_pos(position.0, position.1);
    }

    pub fn get_size_limits(&self) -> (i32, i32, i32, i32) {
        self.size_limits
    }

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

    pub fn swap(&mut self) {
        self.window.swap_buffers();
    }

    pub fn set_vsync(&mut self, vsync: WindowVsyncMode) -> WindowResult<()> {
        let mode = match vsync {
            WindowVsyncMode::Disabled => SwapInterval::None,
            WindowVsyncMode::Enabled => SwapInterval::Sync(1),
            WindowVsyncMode::Adaptive => SwapInterval::Adaptive,
        };
        with_glfw(|glfw| glfw.set_swap_interval(mode))
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

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

    pub fn set_fullscreen_primary(&mut self, preferred_res: (i32, i32)) {
        WindowMonitor::with_primary_monitor(|primary| {
            if let Some(primary) = primary {
                self.set_fullscreen_monitor(&primary, preferred_res)
            }
        });
    }

    pub fn set_fullscreen_monitor(&mut self, monitor: &WindowMonitor, preferred_res: (i32, i32)) {
        if self.fullscreen {
            return;
        }

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

    pub fn set_icon(&mut self, images: Vec<RgbaImage>) {
        self.window.set_icon(images);
    }

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

    pub fn set_cursor_path(&mut self, path: &Path, center: (u32, u32)) -> WindowResult<()> {
        match file_util::path_to_image(path) {
            Ok(img) => self.set_cursor(img.to_rgba8(), center),
            Err(error) => {
                log!(Error, &error, "Couldn't load a cursor");
                return Err(WindowError::IOError(error));
            }
        }
    }

    pub fn reset_cursor(&mut self) {
        self.window.set_cursor(None);
    }

    pub fn make_context_current(&mut self) {
        self.window.make_current();
    }

    pub fn is_focused(&self) -> bool {
        self.window.is_focused()
    }

    pub fn request_focus(&mut self) {
        self.window.focus();
    }

    pub fn get_opacity(&self) -> f32 {
        self.window.get_opacity()
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.window.set_opacity(opacity);
    }

    pub fn is_iconified(&self) -> bool {
        self.window.is_iconified()
    }

    pub fn set_iconified(&mut self, iconified: bool) {
        if iconified {
            self.window.iconify();
        } else {
            self.window.restore();
        }
    }

    pub fn is_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    pub fn set_maximized(&mut self, maximized: bool) {
        if maximized {
            self.window.maximize();
        } else {
            self.window.restore();
        }
    }

    pub fn is_decorated(&self) -> bool {
        self.window.is_decorated()
    }

    pub fn set_decorated(&mut self, decorated: bool) {
        self.window.set_decorated(decorated);
    }

    pub fn is_hovered(&self) -> bool {
        self.window.is_hovered()
    }

    pub fn is_transparent(&self) -> bool {
        self.window.is_framebuffer_transparent()
    }

    pub fn is_debug_context(&self) -> bool {
        self.window.is_opengl_debug_context()
    }

    pub fn poll_events(&mut self) -> WindowResult<()> {
        with_glfw(|glfw| glfw.poll_events())
    }

    pub fn get_framebuffer_size(&self) -> (i32, i32) {
        self.window.get_framebuffer_size()
    }
}
