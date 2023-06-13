use crate::core::game_window::WindowVsyncMode;
use crate::settings::Setting::*;
use crate::utils::logger::Severity;
use crate::utils::logger::Severity::*;
use crate::utils::types::*;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone)]
pub enum Setting {
    Empty,
    Boolean(bool),
    Str(String),
    UnsignedInt(u64),
    SignedInt(i64),
    FloatingPoint(f64),
    IVector2(Vector2i),
    FVector2(Vector2f),
    IVector3(Vector3i),
    FVector3(Vector3f),
    IVector4(Vector4i),
    FVector4(Vector4f),
    LogSeverity(Severity),
    Vsync(WindowVsyncMode),
}

impl Setting {
    pub fn is_empty(&self) -> bool {
        match self {
            Empty => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Boolean(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Str(_) => true,
            _ => false,
        }
    }

    pub fn is_unsigned_int(&self) -> bool {
        match self {
            UnsignedInt(_) => true,
            _ => false,
        }
    }

    pub fn is_signed_int(&self) -> bool {
        match self {
            SignedInt(_) => true,
            _ => false,
        }
    }

    pub fn is_floating_point(&self) -> bool {
        match self {
            FloatingPoint(_) => true,
            _ => false,
        }
    }

    pub fn is_int_vec2(&self) -> bool {
        match self {
            IVector2(_) => true,
            _ => false,
        }
    }

    pub fn is_float_vec2(&self) -> bool {
        match self {
            FVector2(_) => true,
            _ => false,
        }
    }

    pub fn is_int_vec3(&self) -> bool {
        match self {
            IVector3(_) => true,
            _ => false,
        }
    }

    pub fn is_float_vec3(&self) -> bool {
        match self {
            FVector3(_) => true,
            _ => false,
        }
    }

    pub fn is_int_vec4(&self) -> bool {
        match self {
            IVector4(_) => true,
            _ => false,
        }
    }

    pub fn is_float_vec4(&self) -> bool {
        match self {
            FVector4(_) => true,
            _ => false,
        }
    }

    pub fn is_log_severity(&self) -> bool {
        match self {
            LogSeverity(_) => true,
            _ => false,
        }
    }

    pub fn is_vsync_mode(&self) -> bool {
        match self {
            Vsync(_) => true,
            _ => false,
        }
    }

    pub fn as_boolean_or(&self, default: bool) -> bool {
        match self {
            Boolean(value) => *value,
            _ => default,
        }
    }

    pub fn as_string_or<'a>(&'a self, default: &'a String) -> &String {
        match self {
            Str(value) => value,
            _ => default,
        }
    }

    pub fn as_str_or<'a>(&'a self, default: &'a str) -> &str {
        match self {
            Str(value) => value,
            _ => default,
        }
    }

    pub fn as_unsigned_int_or(&self, default: u64) -> u64 {
        match self {
            UnsignedInt(value) => *value,
            _ => default,
        }
    }

    pub fn as_signed_int_or(&self, default: i64) -> i64 {
        match self {
            SignedInt(value) => *value,
            _ => default,
        }
    }

    pub fn as_int_vec2_or(&self, default: Vector2i) -> Vector2i {
        match self {
            IVector2(value) => *value,
            _ => default,
        }
    }

    pub fn as_float_vec2_or(&self, default: Vector2f) -> Vector2f {
        match self {
            FVector2(value) => *value,
            _ => default,
        }
    }

    pub fn as_int_vec3_or(&self, default: Vector3i) -> Vector3i {
        match self {
            IVector3(value) => *value,
            _ => default,
        }
    }

    pub fn as_float_vec3_or(&self, default: Vector3f) -> Vector3f {
        match self {
            FVector3(value) => *value,
            _ => default,
        }
    }

    pub fn as_int_vec4_or(&self, default: Vector4i) -> Vector4i {
        match self {
            IVector4(value) => *value,
            _ => default,
        }
    }

    pub fn as_float_vec4_or(&self, default: Vector4f) -> Vector4f {
        match self {
            FVector4(value) => *value,
            _ => default,
        }
    }

    pub fn as_log_severity_or(&self, default: Severity) -> Severity {
        match self {
            LogSeverity(value) => *value,
            _ => default,
        }
    }

    pub fn as_vsync_mode_or(&self, default: WindowVsyncMode) -> WindowVsyncMode {
        match self {
            Vsync(value) => *value,
            _ => default,
        }
    }
}

pub struct GameSettings {
    settings: RwLockHashMap<String, Setting>,
}

impl GameSettings {
    pub fn new() -> Self {
        let obj = Self {
            settings: RwLock::new(HashMap::new()),
        };

        obj.set("render.openAL", Boolean(true));
        obj.set("render.openGL", Boolean(true));
        obj.set("render.resolution", IVector2(Vector2i::new(1920, 1080))); // Render target resolution
        obj.set("handler.stopTimeout", UnsignedInt(10000)); // 10 s
        obj.set("assets.assetSheet", Str(String::from("/res/main.txt")));
        obj.set(
            "assets.internalSheet",
            Str(String::from("/internal/internal_assets.txt")),
        );
        obj.set("engine.useCurrentThreadAsPrimary", Boolean(false));

        // Game window

        obj.set(
            "window.fullscreenResolution",
            IVector2(Vector2i::new(1920, 1080)),
        ); // Preferred fullscreen resolution
        obj.set("window.size", IVector2(Vector2i::new(256, 256)));
        obj.set("window.minimumSize", IVector2(Vector2i::new(256, 256)));
        obj.set("window.maximumSize", IVector2(Vector2i::new(-1, -1))); // No max size
        obj.set("window.fullscreen", Boolean(false));
        obj.set("window.resizable", Boolean(true));
        obj.set("window.maximized", Boolean(false));
        obj.set("window.vsync", Vsync(WindowVsyncMode::Enabled));
        obj.set("window.transparent", Boolean(false));

        obj.set("window.debugContext", Boolean(true));

        obj.set("window.title", Str(String::from("Spaghetti game")));
        obj.set("window.icon16", Str(String::from("res/icon16.png")));
        obj.set("window.icon32", Str(String::from("res/icon32.png")));

        // Networking
        obj.set("online.port", UnsignedInt(9018));
        obj.set("online.bufferSize", UnsignedInt(1024 * 256)); // 256 KB
        obj.set("online.timeoutTime", UnsignedInt(500000));
        obj.set("online.verifyToken", Boolean(false));
        obj.set("online.maxClients", UnsignedInt(10));
        obj.set("online.maxDisconnections", UnsignedInt(10));
        obj.set("online.awaitTimeout", UnsignedInt(10000));
        obj.set("online.reconnectAttempts", UnsignedInt(10));

        // Logging
        obj.set("log.autoCreate", Boolean(true));
        obj.set("log.printSeverity", LogSeverity(Debug));
        obj.set("log.fileSeverity", LogSeverity(Debug));

        obj
    }

    pub fn set(&self, setting_name: &str, value: Setting) {
        // TODO EVENT DISPATCHER
        let mut map = self.settings.write().unwrap();
        map.insert(setting_name.to_string(), value);
    }

    pub fn get(&self, setting_name: &str) -> Setting {
        let map = self.settings.read().unwrap();
        let option = map.get(setting_name);
        if let Some(setting) = option {
            return setting.clone();
        }
        Empty
    }
}

impl Clone for GameSettings {
    fn clone(&self) -> Self {
        let settings = Self::new();
        {
            // Lock settings
            let mut other_map = settings.settings.write().unwrap();
            let our_map = self.settings.read().unwrap();

            // Copy settings
            other_map.extend(our_map.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        settings
    }
}
