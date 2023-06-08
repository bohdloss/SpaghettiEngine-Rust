use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;
use std::sync::{Arc};
use std::sync::mpsc::Receiver;
use glfw::{Context, Glfw, Monitor, Window, WindowEvent, WindowHint, WindowMode};
use image::{Rgb, RgbaImage};
use crate::settings::GameSettings;
use crate::utils::{file_util, Logger};
use crate::utils::types::Vector2i;

pub struct GameWindow {
    settings: Arc<GameSettings>,
    logger: Arc<Logger>,
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>
}

impl GameWindow {

    pub fn new(settings: Arc<GameSettings>, super_logger: &Arc<Logger>) -> Result<GameWindow, ()> {

        let logger = Logger::from_str(&super_logger, "GameWindow");
        let mut glfw;

        // Initialize glfw
        match glfw::init(glfw::FAIL_ON_ERRORS) {
            Ok(glfw_) => {
                glfw = glfw_;
            },
            Err(error) => {
                logger.print_fatal_err("Error initializing glfw", &error);
                return Err(());
            }
        }

        // Get window settings
        let empty_string = "".to_string();

        let mut size = settings.get("window.size")
            .as_int_vec2_or(Vector2i::new(100, 100));

        let min_size = settings.get("window.minimumSize")
            .as_int_vec2_or(Vector2i::new(1, 1));

        let max_size = settings.get("window.maximumSize")
            .as_int_vec2_or(Vector2i::new(-1, -1)); // Meaning no max size

        let is_fullscreen = settings.get("window.fullscreen")
            .as_boolean_or(false);

        let is_resizable = settings.get("window.resizable")
            .as_boolean_or(true);

        let is_maximized = settings.get("window.maximized")
            .as_boolean_or(true);

        let is_debug_context = settings.get("window.debugContext")
            .as_boolean_or(false);

        let title = settings.get("window.title");
        let title = title.as_string_or(&empty_string);

        let icon16 = settings.get("window.icon16");
        let icon16 = icon16.as_string_or(&empty_string);

        let icon32 = settings.get("window.icon32");
        let icon32 = icon32.as_string_or(&empty_string);

        // Set window hints
        glfw.window_hint(WindowHint::Resizable(is_resizable));
        glfw.window_hint(WindowHint::Maximized(is_maximized));
        glfw.window_hint(WindowHint::Visible(false));
        glfw.window_hint(WindowHint::OpenGlDebugContext(is_debug_context));

        let primary_monitor = Monitor::from_primary();

        let mode = if is_fullscreen {
            if let Some(mode) = primary_monitor.get_video_mode() {
                // Try to get current video mode
                size.x = mode.width as i32;
                size.y = mode.height as i32;

                WindowMode::FullScreen(&primary_monitor)
            } else {
                // Otherwise list all video modes...
                let modes = primary_monitor.get_video_modes();

                let preferred = settings.get("screen.resolution")
                    .as_int_vec2_or(Vector2i::new(1920, 1080));
                size.clone_from(&preferred);

                // ...and search for the preferred one
                let mut found = false;
                for mode in modes.iter() {
                    logger.print_debug(&format!("{}, {}", mode.width, mode.height));

                    if mode.width as i32 == preferred.x && mode.height as i32 == preferred.y {
                        found = true;
                        break;
                    }
                }

                // Did we find it? If not fallback to windowed mode
                if found {
                    WindowMode::FullScreen(&primary_monitor)
                } else {
                    WindowMode::Windowed
                }
            }
        } else {
            WindowMode::Windowed
        };

        // Create window
        let mut window;
        let receiver;
        match glfw.create_window(size.x as u32, size.y as u32, title, mode) {
            Some(window_) => {
                window = window_.0;
                receiver = window_.1;
            },
            None => {
                logger.print_fatal("Error creating glfw window");
                return Err(());
            }
        }

        // Apply some last settings
        window.set_size_limits(if min_size.x > 0 { Some(min_size.x as u32) } else { None },
                               if min_size.y > 0 { Some(min_size.y as u32) } else { None },
                               if max_size.x > 0 { Some(max_size.x as u32) } else { None },
                               if max_size.y > 0 { Some(max_size.y as u32) } else { None });

        let mut icons: Vec<RgbaImage> = Vec::with_capacity(2);

        match file_util::path_to_image(Path::new(icon16)) {
            Ok(img) => {
                icons.push(img.to_rgba8());
            },
            Err(error) => {
                logger.print_error_err("Couldn't load 16x16 icon", &error);
            }
        }
        match file_util::path_to_image(Path::new(icon32)) {
            Ok(img) => {
                icons.push(img.to_rgba8());
            },
            Err(error) => {
                logger.print_error_err("Couldn't load 32x32 icon", &error);
            }
        }

        window.set_icon(icons);

        Ok(Self {
            settings,
            logger,
            glfw,
            window,
            receiver
        })
    }

    pub fn do_loop(&mut self) {
        self.window.show();
        while !self.window.should_close() {
            self.glfw.poll_events();

            for (_, b) in glfw::flush_messages(&self.receiver) {
                match b {
                    WindowEvent::Focus(focused) => {
                        println!("focused {}", focused)
                    },
                    _ => {
                        println!("Unknown event");
                    }
                }
            }

            self.window.swap_buffers();
        }
    }

}