use crate::events::GameEvent;
use crate::utils::id_type::id_type;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::RwLock;

#[macro_export]
macro_rules! register_game_event {
    ($name:ident, data -> $event_data:ident, new -> $constructor:ident) => {
        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn $name() {
            $crate::events::event_registry::register_event_type::<$name>(<$name>::$constructor);
        }

        impl $crate::events::game_event::GameEvent for $name {
            fn get_event_data(&self) -> &$crate::events::game_event::EventData {
                &self.$event_data
            }
            fn get_event_data_mut(&mut self) -> &mut $crate::events::game_event::EventData {
                &mut self.$event_data
            }
            fn get_event_type(&self) -> $crate::events::event_registry::EventType {
                let id = $crate::events::event_registry::get_event_type_of::<$name>();
                id.unwrap()
            }
        }
    };
}

id_type!(EventType);

pub struct EventTypeMetadata {
    pub constructor: fn() -> Box<dyn GameEvent>,
    pub event_type: EventType,
    pub type_id: TypeId,
    pub name: String,
}

static METADATA: RwLock<Vec<EventTypeMetadata>> = RwLock::new(Vec::new());
static TYPE_TO_EVENT_TYPE: Lazy<RwLock<HashMap<TypeId, EventType>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_event_type<T>(constructor: fn() -> Box<dyn GameEvent>) -> EventType
where
    T: GameEvent,
{
    let mut metadata = METADATA.write().unwrap();
    let id = EventType::from(metadata.len() as u64);

    // Construct metadata
    let data = EventTypeMetadata {
        constructor,
        event_type: id,
        type_id: TypeId::of::<T>(),
        name: std::any::type_name::<T>().to_string(),
    };

    metadata.push(data);
    drop(metadata);

    let mut type_to_event_type = TYPE_TO_EVENT_TYPE.write().unwrap();
    type_to_event_type.insert(TypeId::of::<T>(), id);

    id
}

pub fn get_event_type(type_id: &TypeId) -> Option<EventType> {
    TYPE_TO_EVENT_TYPE.read().unwrap().get(type_id).copied()
}

pub fn get_event_type_of<T>() -> Option<EventType>
where
    T: GameEvent,
{
    get_event_type(&TypeId::of::<T>())
}

pub fn get_event_constructor(event_type: EventType) -> Option<fn() -> Box<dyn GameEvent>> {
    METADATA
        .read()
        .unwrap()
        .get(event_type.id as usize)
        .map(|data| data.constructor)
}

pub fn with_event_types<T>(f: T)
where
    T: Fn(&EventTypeMetadata),
{
    for metadata in METADATA.read().unwrap().iter() {
        f(&metadata);
    }
}
