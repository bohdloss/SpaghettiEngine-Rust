use std::collections::HashMap;
use std::sync::{Mutex};
use once_cell::sync::Lazy;
use crate::settings::Setting::*;
use crate::utils::types::*;

pub const PREFIX: Lazy<String> = Lazy::new(|| String::from("spaghetti."));

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
	FVector4(Vector4f)
}

pub struct GameSettings {
	settings: MutexHashMap<String, Setting>
}

impl GameSettings {

	pub fn new() -> Self {
		let mut new = Self {
			settings: Mutex::new(HashMap::new())
		};

		new.set_engine_setting(String::from("render.openAL"), Boolean(true));
		new.set_engine_setting(String::from("render.openGL"), Boolean(true));
		new.set_engine_setting(String::from("screen.resolution"), IVector2(Vector2i::new(256, 256)));
		new.set_engine_setting(String::from("render.resolution"), IVector2(Vector2i::new(256, 256)));
		new.set_engine_setting(String::from("handler.stopTimeout"), UnsignedInt(10000)); // 10 s
		new.set_engine_setting(String::from("assets.assetSheet"), Str(String::from("/res/main.txt")));
		new.set_engine_setting(String::from("assets.internalSheet"), Str(String::from("/internal/internal_assets.txt")));
		new.set_engine_setting(String::from("engine.useCurrentThreadAsPrimary"), Boolean(false));

		// Game window
		new.set_engine_setting(String::from("window.size"), IVector2(Vector2i::new(256, 256)));
		new.set_engine_setting(String::from("window.minimumSize"), IVector2(Vector2i::new(100, 100)));
		new.set_engine_setting(String::from("window.size"), IVector2(Vector2i::new(256, 256)));
		new.set_engine_setting(String::from("window.fullscreen"), Boolean(false));
		new.set_engine_setting(String::from("window.resizable"), Boolean(true));
		new.set_engine_setting(String::from("window.vsync"), Boolean(true));

		new.set_engine_setting(String::from("window.debugContext"), Boolean(true));

		new.set_engine_setting(String::from("window.title"), Str(String::from("Spaghetti game")));
		new.set_engine_setting(String::from("window.icon16"), Str(String::from("/res/icon16.png")));
		new.set_engine_setting(String::from("window.icon32"), Str(String::from("/res/icon32.png")));

		// Networking
		new.set_engine_setting(String::from("online.port"), UnsignedInt(9018));
		new.set_engine_setting(String::from("online.bufferSize"), UnsignedInt(1024 * 256)); // 256 KB
		new.set_engine_setting(String::from("online.timeoutTime"), UnsignedInt(500000));
		new.set_engine_setting(String::from("online.verifyToken"), Boolean(false));
		new.set_engine_setting(String::from("online.maxClients"), UnsignedInt(10));
		new.set_engine_setting(String::from("online.maxDisconnections"), UnsignedInt(10));
		new.set_engine_setting(String::from("online.awaitTimeout"), UnsignedInt(10000));
		new.set_engine_setting(String::from("online.reconnectAttempts"), UnsignedInt(10));

		// Logging
		new.set_engine_setting(String::from("log.autoCreate"), Boolean(true));
		new.set_engine_setting(String::from("log.printSeverity"), UnsignedInt(0)); // TODO LOGGER CONST
		new.set_engine_setting(String::from("log.logSeverity"), UnsignedInt(0)); // TODO LOGGER CONST

		new
	}

	pub fn set_engine_setting(&mut self, setting_name: String, value: Setting) {
		let mut full_name = PREFIX.clone();
		full_name.push_str(&setting_name);
		self.set_setting(full_name, value);
	}

	pub fn get_engine_setting(&mut self, setting_name: String) -> Setting {
		let mut full_name = PREFIX.clone();
		full_name.push_str(&setting_name);
		self.get_setting(&full_name)
	}

	pub fn set_setting(&mut self, setting_name: String, value: Setting) {
		// TODO EVENT DISPATCHER
		if let Ok(mut map) = self.settings.lock() {
			map.insert(setting_name, value);
		} else {
			panic!();
		}
	}

	pub fn get_setting(&self, setting_name: &String) -> Setting {
		{
			let map = self.settings.lock().unwrap();
			let option = map.get(setting_name);
			if let Some(setting) = option {
				return setting.clone();
			}
		}
		Empty
	}

}