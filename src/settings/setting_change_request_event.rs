use crate::events::EventData;
use crate::events::GameEvent;
use crate::register_game_event;
use crate::settings::Setting;
use crate::settings::Setting::Empty;

pub struct SettingChangeRequestEvent {
    event_data: EventData,
    setting_name: String,
    old_value: Setting,
    new_value: Setting,
}

register_game_event!(SettingChangeRequestEvent, data -> event_data, new -> new_empty);

impl SettingChangeRequestEvent {
    pub fn new_empty() -> Box<dyn GameEvent> {
        Box::new(Self {
            event_data: EventData::new(),
            setting_name: String::from(""),
            old_value: Empty,
            new_value: Empty,
        })
    }

    pub fn new(setting_name: String, old_value: Setting, new_value: Setting) -> Box<Self> {
        Box::new(Self {
            event_data: EventData::new(),
            setting_name,
            old_value,
            new_value,
        })
    }

    pub fn get_setting_name(&self) -> &str {
        &self.setting_name
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
