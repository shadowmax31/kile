use wayland_client::Connection;

use crate::client::LayoutManager;

mod client;
mod protocol;

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();

    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut layout_manager = LayoutManager::default();

    event_queue.roundtrip(&mut layout_manager).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut layout_manager).unwrap();
    }
}
