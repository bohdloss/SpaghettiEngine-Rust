use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;
use std::sync::{Arc};
use std::sync::mpsc::Receiver;
use glfw::{Callback, Context, Cursor, Error, ErrorCallback, Glfw, Monitor, SwapInterval, Window, WindowEvent, WindowHint, WindowMode};
use glfw::ffi::glfwSetErrorCallback;
use image::{RgbaImage};
use crate::settings::GameSettings;
use crate::settings::Setting::Str;
use crate::utils::{file_util, Logger};
use crate::utils::types::Vector2i;

fn log_errors(error: Error, description: String, _: &()) {
    Logger::error_err(&format!("GLFW Error: {}", description), &error);
}

static LOG_ERRORS: Option<ErrorCallback<()>>= Some(Callback {
    f: log_errors as fn(Error, String, &()),
    data: ()
});

pub type WindowResult<T> = Result<T, WindowError>;

#[derive(Debug)]
pub enum WindowError {
    CreationError,
    NoVideoMode,
    IOError(io::Error),
    CursorError
}

impl std::error::Error for WindowError {}

impl Display for WindowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::CreationError => write!(f, "CreationError"),
            WindowError::NoVideoMode => write!(f, "NoVideoMode"),
            WindowError::IOError(error) => write!(f, "IOError: {}", error),
            WindowError::CursorError => write!(f, "CursorError")
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum VsyncMode {
    Disabled,
    Enabled,
    Adaptive
}

#[derive(Debug)]
pub struct GameWindow {
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,
    title: String,
    size_limits: (i32, i32, i32, i32),
    fullscreen: bool,
    saved_size: (i32, i32),
}

impl GameWindow {

    pub fn new(settings: &Arc<GameSettings>) -> WindowResult<GameWindow> {
        let mut glfw;

        // Initialize glfw
        match glfw::init(LOG_ERRORS) {
            Ok(glfw_) => {
                glfw = glfw_;
            },
            Err(error) => {
                Logger::fatal_err("Error initializing glfw", &error);
                return Err(WindowError::CreationError);
            }
        }

        // Get window settings
        let empty_string = "".to_string();

        let windowed_size = settings.get("window.size")
            .as_int_vec2_or(Vector2i::new(256, 256));

        let min_size = settings.get("window.minimumSize")
            .as_int_vec2_or(Vector2i::new(64, 64));

        let max_size = settings.get("window.maximumSize")
            .as_int_vec2_or(Vector2i::new(-1, -1)); // Meaning no max size

        let is_fullscreen = settings.get("window.fullscreen")
            .as_boolean_or(false);

        let is_resizable = settings.get("window.resizable")
            .as_boolean_or(true);

        let is_maximized = settings.get("window.maximized")
            .as_boolean_or(true);

        let is_transparent = settings.get("window.transparent")
            .as_boolean_or(false);

        let is_debug_context = settings.get("window.debugContext")
            .as_boolean_or(false);

        let title = settings.get("window.title");
        let title = title.as_string_or(&empty_string);

        let icon32 = if let Str(path) = settings.get("window.icon32")
        { Some(path) } else { None };
        let icon16 = if let Str(path) = settings.get("window.icon16")
        { Some(path) } else { None };

        // Set window hints
        glfw.window_hint(WindowHint::Resizable(is_resizable));
        glfw.window_hint(WindowHint::Maximized(is_maximized));
        glfw.window_hint(WindowHint::Visible(false));
        glfw.window_hint(WindowHint::OpenGlDebugContext(is_debug_context));
        glfw.window_hint(WindowHint::TransparentFramebuffer(is_transparent));

        // Get fullscreen info
        let primary_monitor = Monitor::from_primary();
        let fullscreen_size = settings.get("window.fullscreenResolution")
            .as_int_vec2_or(Vector2i::new(1920, 1080));

        // Determine window mode...
        let mode = if is_fullscreen {
            WindowMode::FullScreen(&primary_monitor)
        } else {
            WindowMode::Windowed
        };

        // ...and size
        let size = if is_fullscreen {
            (if fullscreen_size.x > 0 { fullscreen_size.x } else { 1 },
             if fullscreen_size.y > 0 { fullscreen_size.y } else { 1 })
        } else {
            (if windowed_size.x > 0 { windowed_size.x } else { 1 },
             if windowed_size.y > 0 { windowed_size.y } else { 1 })
        };

        // Create glfw window
        let window;
        let receiver;
        match glfw.create_window(size.0 as u32, size.1 as u32, title, mode) {
            Some(window_) => {
                window = window_.0;
                receiver = window_.1;
            },
            None => {
                Logger::fatal("Error creating glfw window");
                return Err(WindowError::CreationError);
            }
        }

        // Construct game window object
        let mut game_window = Self {
            glfw,
            window,
            receiver,
            title: title.clone(),
            size_limits: (0, 0, 0, 0),
            fullscreen: if let WindowMode::Windowed = mode { false } else { true },
            saved_size: (windowed_size.x, windowed_size.y)
        };

        // Apply some last settings
        game_window.set_size_limits((min_size.x, min_size.y, max_size.x, max_size.y));

	    Logger::debug(&format!("{}, {}", size.0, size.1));
        game_window.set_size(size);

        let mut paths: Vec<&Path> = Vec::new();
        if let Some(path) = &icon16 {
            paths.push(Path::new(path));
        }
        if let Some(path) = &icon32 {
            paths.push(Path::new(path));
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

        self.window.set_size(if size.0 > 0 { size.0 } else { 1 },
                             if size.1 > 0 { size.1 } else { 1 });
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
        self.window.set_size_limits(if size_limits.0 > 0 { Some(size_limits.0 as u32) } else { None },
                               if size_limits.1 > 0 { Some(size_limits.1 as u32) } else { None },
                               if size_limits.2 > 0 { Some(size_limits.2 as u32) } else { None },
                               if size_limits.3 > 0 { Some(size_limits.3 as u32) } else { None });

        // Validate size limits
        let min_valid = size_limits.0 > 0 && size_limits.1 > 0;
        let max_valid = size_limits.2 > 0 && size_limits.3 > 0;

        self.size_limits = (if min_valid { size_limits.0 } else { -1 },
                            if min_valid { size_limits.1 } else { -1 },
                            if max_valid { size_limits.2 } else { -1 },
                            if max_valid { size_limits.3 } else { -1 });
    }

    pub fn swap(&mut self) {
        self.window.swap_buffers();
    }

    pub fn set_vsync(&mut self, vsync: VsyncMode) {
        let mode = match vsync {
            VsyncMode::Disabled => SwapInterval::None,
            VsyncMode::Enabled => SwapInterval::Sync(1),
            VsyncMode::Adaptive => SwapInterval::Adaptive
        };
        self.glfw.set_swap_interval(mode);
    }

    fn calc_fullscreen_info(monitor: &Monitor, preferred_res: (i32, i32)) -> WindowResult<(i32, i32, Option<u32>)> {
        // Get preferred resolution
        let mut actual: (i32, i32) = (preferred_res.0, preferred_res.1);
        let mut refresh_rate: Option<u32> = None;

        // Search through available video modes for one with the preferred resolution
        let modes = monitor.get_video_modes();
        let mut found = false;
        for mode in modes.iter() {
            if mode.width as i32 == preferred_res.0 && mode.height as i32 == preferred_res.1 {
                found = true;
                refresh_rate = Some(mode.refresh_rate);
                break;
            }
        }

        // If we didnt find it, try to use the current video mode
        if !found {
            if let Some(mode) = monitor.get_video_mode() {
                actual = (mode.width as i32, mode.height as i32);
                refresh_rate = Some(mode.refresh_rate);
            } else {
                Logger::error("Couldn't find any suitable video mode when switching to fullscreen");
                return Err(WindowError::NoVideoMode);
            }
        }

        Ok((actual.0, actual.1, refresh_rate))
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool, preferred_res: (i32, i32)) -> WindowResult<()> {
        if self.fullscreen == fullscreen {
            return Ok(());
        }

        let size: (i32, i32);
        let monitor = Monitor::from_window(&self.window);
        let mode_size: (i32, i32);
        let mode_refresh_rate: Option<u32>;
        let window_mode: WindowMode;

        if fullscreen {

            let info = Self::calc_fullscreen_info(&monitor, preferred_res)?;

            size = (info.0, info.1);
            mode_size = (info.0, info.1);
            mode_refresh_rate = info.2;
            window_mode = WindowMode::FullScreen(&monitor);
        } else {

            // Restore the saved size
            size = self.saved_size;
            mode_size = if let Some(mode) = monitor.get_video_mode() {
                (mode.width as i32, mode.height as i32)
            } else {
                (0, 0)
            };
            mode_refresh_rate = None;
            window_mode = WindowMode::Windowed;
        }

        // Calculate centered position
        let position = (mode_size.0 / 2 - size.0 / 2,
                        mode_size.1 / 2 - size.1 / 2);

        self.window.set_monitor(window_mode,
                                position.0,
                                position.1,
                                size.0 as u32,
                                size.1 as u32,
                                mode_refresh_rate);
        self.fullscreen = fullscreen;
        Ok(())
    }

    pub fn center(&mut self) {
        if let Some(mode) = Monitor::from_window(&self.window).get_video_mode() {
            let size = self.get_size();
            self.set_position((mode.width as i32 / 2 - size.0 / 2,
                                mode.height as i32 / 2 - size.1 / 2));
        }
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
                },
                Err(error) => {
                    Logger::error_err("Couldn't load a window icon", &error);
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
            Logger::error("Cursor size cannot be larger than 256x256");
            return Err(WindowError::CursorError);
        }

        let cursor = Cursor::create(cursor, center.0, center.1);
        self.window.set_cursor(Some(cursor));
        Ok(())
    }

    pub fn set_cursor_path(&mut self, path: &Path, center: (u32, u32)) -> WindowResult<()> {
        match file_util::path_to_image(path) {
            Ok(img) => {
                self.set_cursor(img.to_rgba8(), center)
            },
            Err(error) => {
                Logger::error_err("Couldn't load a cursor", &error);
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

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

	pub fn get_framebuffer_size(&self) -> (i32, i32) {
		self.window.get_framebuffer_size()
	}

}