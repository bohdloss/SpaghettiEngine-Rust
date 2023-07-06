use std::sync::Arc;
use glfw::{Cursor, CursorMode, Monitor, RenderContext, SwapInterval, VidMode, WindowHint};
use image::RgbaImage;
use crate::utils::new_empty::NewEmpty;
use crate::utils::request_pipe::RequestPipe;
use crate::window::window_manager::WindowError;

pub(super) enum WindowPacket {
    WOk,
    WError(WindowError),
    RequestPosition,
    Position(i32, i32),
    RequestResizable,
    Resizable(bool),
    RequestSize,
    Size(i32, i32),
    RequestVisible,
    Visible(bool),
    Title(String),
    SizeLimits(Option<u32>, Option<u32>, Option<u32>, Option<u32>),
    Icon(Vec<RgbaImage>),
    SetCursor(Option<Cursor>),
    SetCursorMode(CursorMode),
    RequestFocused,
    Focused(bool),
    RequestOpacity,
    Opacity(f32),
    RequestIconified,
    Iconified(bool),
    RequestMaximized,
    Maximized(bool),
    RequestDecorated,
    Decorated(bool),
    RequestHovered,
    Hovered(bool),
    RequestTransparent,
    Transparent(bool),
    RequestDebugContext,
    DebugContext(bool),
    RequestFrameBufferSize,
    FrameBufferSize(i32, i32),
    SetMonitor(Option<Monitor>, i32, i32, u32, u32, Option<u32>)
}

impl NewEmpty for WindowPacket {
    fn new_empty() -> Self {
        Self::WOk
    }
}

pub(super) enum GlfwPacket {
    GOk,
    GError(WindowError),
    SetDefaultHints,
    SetWindowHint(WindowHint),
    CreateWindow(u32, u32, String),
    WindowCreated(Arc<RequestPipe<WindowPacket>>, RenderContext),
    DestroyWindow(Arc<RequestPipe<WindowPacket>>),
    SetSwapInterval(SwapInterval),
    CreateCursor(RgbaImage, u32, u32),
    CursorCreated(Cursor),
    RequestConnectedMonitors,
    ConnectedMonitors(Vec<Monitor>),
    RequestPrimaryMonitor,
    PrimaryMonitor(Option<Monitor>),
    RequestCurrentVideoMode(Monitor),
    CurrentVideoMode(Option<VidMode>),
    RequestVideoModes(Monitor),
    VideoModes(Vec<VidMode>),
    RequestVirtualPos(Monitor),
    VirtualPos(i32, i32),
    RequestPhysicalSize(Monitor),
    PhysicalSize(i32, i32),
    RequestContentScale(Monitor),
    ContentScale(f32, f32),
    RequestWorkArea(Monitor),
    WorkArea(i32, i32, i32, i32),
    RequestName(Monitor),
    Name(Option<String>),
    Gamma(Monitor, f32)
}

impl NewEmpty for GlfwPacket {
    fn new_empty() -> Self {
        Self::GOk
    }
}