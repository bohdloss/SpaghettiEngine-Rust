use crate::events::event_registry;

#[test]
fn event_registry() {
    println!("All registered event types: ");
    event_registry::with_event_types(|data| {
        println!("Event name: {}", data.name);
    })
}
