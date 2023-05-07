use spaghetti_engine_derive::{AsAny, GameEvent};
use crate::events::game_event::EventData;
use crate::events::GameEvent;
use crate::settings;
use crate::settings::Setting;
use crate::utils::AsAny;

#[derive(GameEvent, AsAny)]
pub struct SettingChangeRequestEvent {
	event: EventData,
	setting_name: String,
	old_value: Setting,
	new_value: Setting
}

impl SettingChangeRequestEvent {

	pub fn get_setting_name(&self) -> &str {
		&self.setting_name
	}

	pub fn get_engine_setting_name(&self) -> &str {
		&self.setting_name[settings::game_settings::PREFIX.len()..]
	}

	pub fn get_old_value(&self) -> &Setting {
		&self.old_value
	}

	pub fn get_new_value(&self) -> &Setting {
		&self.new_value
	}

	pub fn set_new_value(&mut self, value: Setting) {
		self.new_value = value;
	}

}