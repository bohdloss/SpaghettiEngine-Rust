use crate::events::event_registry;
use crate::log;

#[test]
fn event_registry() {
    log!(Debug, "All registered event types: ");
    event_registry::with_event_types(|data| {
        log!(Debug, "Event name: {}", data.name);
    })
}
