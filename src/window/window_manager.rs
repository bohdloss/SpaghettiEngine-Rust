use crate::core::{entry_point, Game};
use crate::input::game_pad_events::{GamePadConnectedEvent, GamePadDisconnectedEvent};
use crate::input::input_dispatcher::NUM_GAME_PADS;
use crate::input::mouse::MouseAxis;
use crate::input::InputDispatcher;
use crate::log;
use crate::utils::new_empty::NewEmpty;
use crate::utils::request_pipe::RequestPipe;
use crate::window::input_transform::*;
use crate::window::packets::GlfwPacket::*;
use crate::window::packets::WindowPacket::*;
use crate::window::packets::{GlfwPacket, WindowPacket};
use crate::window::window_manager::WindowError::*;
use glfw::Action::*;
use glfw::{
    ffi, Cursor, GamepadAxis, GamepadButton, Glfw, JoystickEvent, Monitor, Window, WindowEvent,
    WindowMode,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{mem, ptr, sync};

static WINDOW_LIST: Mutex<Vec<WindowEntry>> = Mutex::new(Vec::new());
pub(super) static GLFW: Lazy<RequestPipe<GlfwPacket>> = Lazy::new(|| RequestPipe::new());

unsafe impl<T: NewEmpty> Send for RequestPipe<T> {}
unsafe impl<T: NewEmpty> Sync for RequestPipe<T> {}

pub(super) trait CustomClone {
    fn custom_clone(&self) -> Self;
}

impl CustomClone for Monitor {
    fn custom_clone(&self) -> Self {
        unsafe {
            let mut clone: Self = mem::zeroed();
            ptr::copy(self, &mut clone, 1);
            clone
        }
    }
}

struct WindowEntry {
    pipe: Arc<RequestPipe<WindowPacket>>,
    window: Box<Window>,
    receiver: Receiver<(f64, WindowEvent)>,
    input_dispatcher: sync::Weak<InputDispatcher>,
}

unsafe impl Send for WindowEntry {}

fn with_handle<T>(window_handle: &RequestPipe<WindowPacket>, f: T)
where
    T: FnOnce(&mut WindowEntry),
{
    let mut window_list = WINDOW_LIST.lock().unwrap();
    let idx = window_list
        .iter()
        .position(|elem| elem.pipe.as_ref() == window_handle);

    if let Some(idx) = idx {
        if let Some(entry) = window_list.get_mut(idx) {
            f(entry);
        }
    }
}

fn register_window(
    window: Box<Window>,
    receiver: Receiver<(f64, WindowEvent)>,
) -> Arc<RequestPipe<WindowPacket>> {
    let window_pipe = Arc::new(RequestPipe::new());
    WINDOW_LIST.lock().unwrap().push(WindowEntry {
        pipe: window_pipe.clone(),
        window,
        receiver,
        input_dispatcher: sync::Weak::new(),
    });
    window_pipe
}

pub(super) fn register_input(
    window_handle: &RequestPipe<WindowPacket>,
    input_dispatcher: sync::Weak<InputDispatcher>,
) {
    with_handle(window_handle, |entry| {
        entry.input_dispatcher = input_dispatcher
    });
}

pub(super) fn unregister_input(window_handle: &RequestPipe<WindowPacket>) {
    with_handle(window_handle, |entry| {
        entry.input_dispatcher = sync::Weak::new()
    });
}

pub(super) fn maybe_init_glfw() -> WindowResult<()> {
    if is_glfw_initialized() {
        return Ok(());
    }
    entry_point::send_event(|| {
        initialize_glfw();

        // Register tasks and shutdown events
        entry_point::register_task(update);
        entry_point::register_shutdown_event(shutdown_glfw);

        Ok(())
    })
}

fn update() {
    let _ = with_glfw(|glfw| {
        glfw.poll_events();

        handle_glfw_request(glfw);

        let mut window_list = WINDOW_LIST.lock().unwrap();
        for entry in window_list.iter_mut() {
            let window = &mut entry.window;

            // Handle requests
            handle_window_request(entry.pipe.as_ref(), window);

            // Handle input?
            if let Some(dispatcher) = entry.input_dispatcher.upgrade() {
                handle_input_update(glfw, entry, &*dispatcher);
            }
        }
    });
}

fn handle_input_update(glfw: &mut Glfw, entry: &mut WindowEntry, dispatcher: &InputDispatcher) {
    let keyboard_state = dispatcher.keyboard_state();
    let mouse_state = dispatcher.mouse_state();

    for index in 0..NUM_GAME_PADS {
        handle_game_pad_input(glfw, dispatcher, index);
    }

    for (_, event) in glfw::flush_messages(&entry.receiver) {
        match event {
            WindowEvent::Key(key, _, action, _) => match action {
                Press => keyboard_state.keys[from_key(key).index()] = true,
                Release => keyboard_state.keys[from_key(key).index()] = false,
                _ => {}
            },
            WindowEvent::MouseButton(button, action, _) => match action {
                Press => mouse_state.buttons[from_mouse_button(button).index()] = true,
                Release => mouse_state.buttons[from_mouse_button(button).index()] = false,
                _ => {}
            },
            WindowEvent::CursorPos(x, y) => {
                mouse_state.axis[MouseAxis::X.index()] = x;
                mouse_state.axis[MouseAxis::Y.index()] = y;
            }
            WindowEvent::Scroll(x, y) => {
                mouse_state.axis[MouseAxis::WheelX.index()] = x;
                mouse_state.axis[MouseAxis::WheelY.index()] = y;
            }
            _ => {}
        }
    }
}

fn handle_game_pad_input(glfw: &mut Glfw, dispatcher: &InputDispatcher, index: usize) {
    let game_pad_state = dispatcher.game_pad_state(index);
    let joystick_id = index_to_joystick(index);
    let joystick = glfw.get_joystick(joystick_id);

    if !joystick.is_present() || !joystick.is_gamepad() {
        return;
    }

    let game_pad = joystick.get_gamepad_state();
    if game_pad.is_none() {
        return;
    }
    let game_pad = game_pad.unwrap();

    for button in 0..ffi::GAMEPAD_BUTTON_LAST {
        if let Some(button) = GamepadButton::from_i32(button as i32) {
            match game_pad.get_button_state(button) {
                Press => game_pad_state.buttons[from_game_pad_button(button).index()] = true,
                Release => game_pad_state.buttons[from_game_pad_button(button).index()] = false,
                _ => {}
            }
        }
    }
    for axis in 0..ffi::GAMEPAD_AXIS_LAST {
        if let Some(axis) = GamepadAxis::from_i32(axis as i32) {
            game_pad_state.axis[from_game_pad_axis(axis).index()] = game_pad.get_axis(axis) as f64;
        }
    }
}

fn handle_window_request(pipe: &RequestPipe<WindowPacket>, window: &mut Window) {
    let _ = pipe.receive().is_some_and(|packet| {
        match packet {
            RequestPosition => {
                let position = window.get_pos();
                pipe.respond(Position(position.0, position.1));
            }
            Position(x, y) => {
                window.set_pos(x, y);
                pipe.respond(WOk);
            }
            RequestResizable => {
                let resizable = window.is_resizable();
                pipe.respond(Resizable(resizable));
            }
            Resizable(resizable) => {
                window.set_resizable(resizable);
                pipe.respond(WOk);
            }
            RequestSize => {
                let size = window.get_size();
                pipe.respond(Size(size.0, size.1));
            }
            Size(width, height) => {
                window.set_size(width, height);
                pipe.respond(WOk);
            }
            RequestVisible => {
                let visible = window.is_visible();
                pipe.respond(Visible(visible));
            }
            Visible(visible) => {
                if visible {
                    window.show();
                } else {
                    window.hide();
                }
                pipe.respond(WOk);
            }
            Title(title) => {
                window.set_title(&title);
                pipe.respond(WOk);
            }
            SizeLimits(min_width, min_height, max_width, max_height) => {
                window.set_size_limits(min_width, min_height, max_width, max_height);
                pipe.respond(WOk);
            }
            Icon(icon) => {
                window.set_icon(icon);
                pipe.respond(WOk);
            }
            SetCursor(cursor) => {
                window.set_cursor(cursor);
                pipe.respond(WOk);
            }
            SetCursorMode(mode) => {
                window.set_cursor_mode(mode);
                pipe.respond(WOk);
            }
            RequestFocused => {
                let focused = window.is_focused();
                pipe.respond(Focused(focused));
            }
            Focused(_) => {
                window.focus();
                pipe.respond(WOk);
            }
            RequestOpacity => {
                let opacity = window.get_opacity();
                pipe.respond(Opacity(opacity));
            }
            Opacity(opacity) => {
                window.set_opacity(opacity);
                pipe.respond(WOk);
            }
            RequestIconified => {
                let iconified = window.is_iconified();
                pipe.respond(Iconified(iconified));
            }
            Iconified(iconified) => {
                if iconified {
                    window.iconify();
                } else if window.is_iconified() {
                    window.restore();
                }
                pipe.respond(WOk);
            }
            RequestMaximized => {
                let maximized = window.is_maximized();
                pipe.respond(Maximized(maximized));
            }
            Maximized(maximized) => {
                if maximized {
                    window.maximize();
                } else if window.is_maximized() {
                    window.restore();
                }
                pipe.respond(WOk);
            }
            RequestDecorated => {
                let decorated = window.is_decorated();
                pipe.respond(Decorated(decorated));
            }
            Decorated(decorated) => {
                window.set_decorated(decorated);
                pipe.respond(WOk);
            }
            RequestHovered => {
                let hovered = window.is_hovered();
                pipe.respond(Hovered(hovered));
            }
            RequestTransparent => {
                let transparent = window.is_framebuffer_transparent();
                pipe.respond(Transparent(transparent));
            }
            RequestDebugContext => {
                let debug_context = window.is_opengl_debug_context();
                pipe.respond(DebugContext(debug_context));
            }
            RequestFrameBufferSize => {
                let frame_buffer_size = window.get_framebuffer_size();
                pipe.respond(FrameBufferSize(frame_buffer_size.0, frame_buffer_size.1));
            }
            SetMonitor(mode, x, y, width, height, refresh_rate) => {
                let mode = match mode {
                    Some(ref monitor) => WindowMode::FullScreen(monitor),
                    None => WindowMode::Windowed,
                };
                window.set_monitor(mode, x, y, width, height, refresh_rate);
                pipe.respond(WOk)
            }
            _ => {
                pipe.respond(WError(InvalidEnum));
            }
        }
        false
    });
}

fn handle_glfw_request(glfw: &mut Glfw) {
    let _ = GLFW.receive().is_some_and(|packet| {
        match packet {
            SetDefaultHints => {
                glfw.default_window_hints();
                GLFW.respond(GOk);
            }
            SetWindowHint(hint) => {
                glfw.window_hint(hint);
                GLFW.respond(GOk);
            }
            CreateWindow(width, height, title) => {
                match glfw.create_window(width, height, &title, WindowMode::Windowed) {
                    Some((mut window, receiver)) => {
                        // Create and register window
                        let render_context = window.render_context();

                        // Set up polling
                        window.set_key_polling(true);
                        window.set_mouse_button_polling(true);
                        window.set_cursor_pos_polling(true);
                        window.set_scroll_polling(true);

                        let window_pipe = register_window(window, receiver);

                        GLFW.respond(WindowCreated(window_pipe, render_context));
                    }
                    None => {
                        GLFW.respond(GError(CreationError));
                    }
                }
            }
            DestroyWindow(window_handle) => {
                let mut window_list = WINDOW_LIST.lock().unwrap();
                let idx = window_list
                    .iter()
                    .position(|elem| elem.pipe == window_handle);

                if let Some(idx) = idx {
                    window_list.remove(idx);
                }
                GLFW.respond(GOk);
            }
            SetSwapInterval(interval) => {
                glfw.set_swap_interval(interval);
                GLFW.respond(GOk);
            }
            CreateCursor(image, x, y) => {
                let cursor = Cursor::create(image, x, y);
                GLFW.respond(CursorCreated(cursor));
            }
            RequestConnectedMonitors => {
                let monitors = glfw.with_connected_monitors(|_, monitors| {
                    let mut cloned_vec: Vec<Monitor> = Vec::new();
                    for monitor in monitors {
                        cloned_vec.push(monitor.custom_clone());
                    }
                    cloned_vec
                });
                GLFW.respond(ConnectedMonitors(monitors));
            }
            RequestPrimaryMonitor => {
                let monitor = glfw.with_primary_monitor(|_, monitor| match monitor {
                    Some(monitor) => Some(monitor.custom_clone()),
                    None => None,
                });
                GLFW.respond(PrimaryMonitor(monitor));
            }
            RequestCurrentVideoMode(monitor) => {
                let video_mode = monitor.get_video_mode();
                GLFW.respond(CurrentVideoMode(video_mode));
            }
            RequestVideoModes(monitor) => {
                let video_modes = monitor.get_video_modes();
                GLFW.respond(VideoModes(video_modes));
            }
            RequestVirtualPos(monitor) => {
                let virtual_pos = monitor.get_pos();
                GLFW.respond(VirtualPos(virtual_pos.0, virtual_pos.1));
            }
            RequestPhysicalSize(monitor) => {
                let physical_size = monitor.get_physical_size();
                GLFW.respond(PhysicalSize(physical_size.0, physical_size.1));
            }
            RequestContentScale(monitor) => {
                let content_scale = monitor.get_content_scale();
                GLFW.respond(ContentScale(content_scale.0, content_scale.1));
            }
            RequestWorkArea(monitor) => {
                let work_area = monitor.get_workarea();
                GLFW.respond(WorkArea(work_area.0, work_area.1, work_area.2, work_area.3));
            }
            RequestName(monitor) => {
                let name = monitor.get_name();
                GLFW.respond(Name(name));
            }
            Gamma(mut monitor, gamma) => {
                monitor.set_gamma(gamma);
                GLFW.respond(GOk);
            }
            _ => {
                GLFW.respond(GError(InvalidEnum));
            }
        }
        false
    });
}

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
    static GLFW_CONTAINER: RefCell<GlfwContainer> = {
        RefCell::new(
            GlfwContainer {
                glfw: None,
                warned: false,
            }
        )
    };
}

fn is_glfw_initialized() -> bool {
    *GLFW_INITIALIZED.lock().unwrap()
}

fn set_glfw_initialized(initialized: bool) -> bool {
    let mut value = GLFW_INITIALIZED.lock().unwrap();
    mem::replace(&mut *value, initialized)
}

fn initialize_glfw() {
    GLFW_CONTAINER.with(|cell| match cell.try_borrow_mut() {
        Ok(mut container) => {
            // Check if already initialized
            if set_glfw_initialized(true) {
                log!(Warning, "Tried to initialize GLFW multiple times");
                return;
            }

            // Initialize glfw
            *container = match glfw::init(|error, description| {
                log!(Error, &error, "{}", description);
            }) {
                Ok(mut glfw) => {
                    log!(
                        Info,
                        "Initialized GLFW version {}",
                        glfw::get_version_string()
                    );
                    glfw.set_joystick_callback(|joystick, event| {
                        if let Some(game) = Game::get_instance().upgrade() {
                            let index = joystick as usize;
                            if let JoystickEvent::Connected = event {
                                game.get_event_dispatcher()
                                    .raise_event(GamePadConnectedEvent::new(index), true);
                            } else {
                                game.get_event_dispatcher()
                                    .raise_event(GamePadDisconnectedEvent::new(index), true);
                            }
                        }
                    });
                    GlfwContainer::new_success(glfw)
                }
                Err(error) => {
                    log!(Fatal, &error, "Glfw initialization failed with error");
                    GlfwContainer::new_failure()
                }
            };
        }
        Err(_) => {}
    });
}

fn shutdown_glfw() {
    GLFW_CONTAINER.with(|cell| match cell.try_borrow_mut() {
        Ok(mut container) => {
            if !is_glfw_initialized() {
                log!(
                    Warning,
                    "Tried to shut down GLFW but it was not initialized"
                );
                return;
            }

            // Drops the previous value shutting down glfw
            *container = GlfwContainer {
                glfw: None,
                warned: false,
            };

            // Set flag to false
            set_glfw_initialized(false);

            log!(Info, "GLFW terminated");
        }
        Err(_) => {}
    });
}

fn with_glfw<T, R>(f: T) -> Result<R, WindowError>
where
    T: FnOnce(&mut Glfw) -> R,
{
    GLFW_CONTAINER.with(|cell| match cell.try_borrow_mut() {
        Ok(mut container) => match container.glfw {
            Some(ref mut glfw) => Ok(f(glfw)),
            None => {
                if !container.was_warned() {
                    log!(Fatal, "No Glfw instance in this thread");
                }
                Err(InternalError)
            }
        },
        Err(error) => {
            log!(Fatal, &error, "Window internal error");
            Err(InternalError)
        }
    })
}

pub type WindowResult<T, E = WindowError> = Result<T, E>;

#[derive(Debug)]
pub enum WindowError {
    InternalError,
    CreationError,
    IOError(std::io::Error),
    CursorError,
    NoMonitor,
    InvalidEnum,
}

impl std::error::Error for WindowError {}

impl Display for WindowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalError => write!(f, "InternalError"),
            CreationError => write!(f, "CreationError"),
            IOError(error) => write!(f, "IOError: {}", error),
            CursorError => write!(f, "CursorError"),
            NoMonitor => write!(f, "NoMonitor"),
            InvalidEnum => write!(f, "InvalidEnum"),
        }
    }
}
