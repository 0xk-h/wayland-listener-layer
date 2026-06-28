mod app;
mod dnd;
mod handlers;
mod utils;

use app::App;
use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell},
        WaylandSurface,
    },
    shm::Shm,
};
use wayland_client::{
    globals::registry_queue_init, 
    protocol::wl_data_device_manager::WlDataDeviceManager, 
    Connection
};

fn main() {
    let conn = Connection::connect_to_env().expect("failed to connect to wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("failed to init registry");
    let qh = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh).expect("wl_compositor missing");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm missing");
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell missing");
    
    let data_device_manager = globals.bind::<WlDataDeviceManager, _, _>(&qh, 3..=3, ()).ok();

    let surface = compositor_state.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Bottom,
        Some("wallpaper-dnd"),
        None,
    );

    layer_surface.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.set_size(0, 0);
    layer_surface.commit();

    let mut app = App::new(
        RegistryState::new(&globals),
        output_state,
        shm,
        seat_state,
        layer_surface,
        data_device_manager,
    );

    event_queue.roundtrip(&mut app).unwrap();

    // Force data device creation if the seat already existed at startup.
    if app.data_device.is_none() {
        if let Some(mgr) = &app.data_device_manager {
            if let Some(seat) = app.seat_state.seats().next() {
                app.data_device = Some(mgr.get_data_device(&seat, &qh, ()));
            }
        }
    }

    // Second roundtrip flushes the data device registration to the compositor
    event_queue.roundtrip(&mut app).unwrap();

    let mut event_loop: EventLoop<App> = EventLoop::try_new().expect("failed to create event loop");
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .expect("failed to insert wayland source");

    loop {
        event_loop.dispatch(None, &mut app).expect("event loop error");
    }
}