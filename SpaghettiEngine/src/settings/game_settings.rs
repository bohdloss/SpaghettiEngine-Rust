use std::collections::HashMap;
use std::sync::RwLock;
use crate::settings::Setting::*;
use crate::utils::logger::Severity;
use crate::utils::logger::Severity::*;
use crate::utils::types::*;

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
	LogSeverity(Severity)
}

pub struct GameSettings {
	settings: RwLockHashMap<String, Setting>,
}

impl GameSettings {

	pub fn new() -> Self {
		let new = Self {
			settings: RwLock::new(HashMap::new()),
		};

		new.set("render.openAL", Boolean(true));
		new.set("render.openGL", Boolean(true));
		new.set("screen.resolution", IVector2(Vector2i::new(256, 256)));
		new.set("render.resolution", IVector2(Vector2i::new(256, 256)));
		new.set("handler.stopTimeout", UnsignedInt(10000)); // 10 s
		new.set("assets.assetSheet", Str(String::from("/res/main.txt")));
		new.set("assets.internalSheet", Str(String::from("/internal/internal_assets.txt")));
		new.set("engine.useCurrentThreadAsPrimary", Boolean(false));

		// Game window
		new.set("window.size", IVector2(Vector2i::new(256, 256)));
		new.set("window.minimumSize", IVector2(Vector2i::new(100, 100)));
		new.set("window.size", IVector2(Vector2i::new(256, 256)));
		new.set("window.fullscreen", Boolean(false));
		new.set("window.resizable", Boolean(true));
		new.set("window.vsync", Boolean(true));

		new.set("window.debugContext", Boolean(true));

		new.set("window.title", Str(String::from("Spaghetti game")));
		new.set("window.icon16", Str(String::from("/res/icon16.png")));
		new.set("window.icon32", Str(String::from("/res/icon32.png")));

		// Networking
		new.set("online.port", UnsignedInt(9018));
		new.set("online.bufferSize", UnsignedInt(1024 * 256)); // 256 KB
		new.set("online.timeoutTime", UnsignedInt(500000));
		new.set("online.verifyToken", Boolean(false));
		new.set("online.maxClients", UnsignedInt(10));
		new.set("online.maxDisconnections", UnsignedInt(10));
		new.set("online.awaitTimeout", UnsignedInt(10000));
		new.set("online.reconnectAttempts", UnsignedInt(10));

		// Logging
		new.set("log.autoCreate", Boolean(true));
		new.set("log.printSeverity", LogSeverity(INFO)); // TODO LOGGER CONST
		new.set("log.fileSeverity", LogSeverity(DEBUG)); // TODO LOGGER CONST

		new
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