use crate::app::App;

use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::PointerHandler,
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_data_device_manager::WlDataDeviceManager,
        wl_output::{self, WlOutput},
        wl_pointer::WlPointer,
        wl_seat::WlSeat,
        wl_shm::Format,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, QueueHandle,
};

// Layer Shell

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = (configure.new_size.0.max(1), configure.new_size.1.max(1));

        if self.pool.is_none() {
            self.pool = Some(
                SlotPool::new((w * h * 4) as usize, &self.shm)
                    .expect("failed to create shm pool"),
            );
        }

        let pool = self.pool.as_mut().unwrap();
        // let (buffer, canvas) = pool
        //     .create_buffer(1, 1, 4, Format::Argb8888)
        //     .expect("failed to create buffer");

        // was: pool.create_buffer(1, 1, 4, Format::Argb8888)
        let (buffer, canvas) = pool
            .create_buffer(w as i32, h as i32, (w * 4) as i32, Format::Argb8888)
            .expect("failed to create buffer");

        // Fill entire canvas red (ARGB8888 byte order: B=0, G=0, R=255, A=255)
        for chunk in canvas.chunks_exact_mut(4) {
            chunk[0] = 0;   // B
            chunk[1] = 0;   // G
            chunk[2] = 255; // R
            chunk[3] = 255; // A
        }

        self.layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface.wl_surface().commit();
        self.configured = true;
    }
}

// Register Seat

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, seat: WlSeat) {
        if let Some(mgr) = &self.data_device_manager {
            self.data_device = Some(mgr.get_data_device(&seat, qh, ()));
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            let _pointer = self.seat_state.get_pointer(qh, &seat).expect("failed to get pointer");
        }
    }

    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat, _: Capability) {}
}

// wl_data_device_manager - no events

impl Dispatch<WlDataDeviceManager, ()> for App {
    fn event(
        _: &mut Self,
        _: &WlDataDeviceManager,
        _: wayland_client::protocol::wl_data_device_manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {}
}

// wl_buffer - ignores events

impl Dispatch<WlBuffer, ()> for App {
    fn event(
        _: &mut Self,
        _: &WlBuffer,
        _: wayland_client::protocol::wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {}
}

// Compositor

impl CompositorHandler for App {
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: &WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: &WlOutput) {}
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: i32) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: u32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlSurface, _: wl_output::Transform) {}
}

// Output

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
}

// Shm

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

// Registry

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

// Pointer

impl PointerHandler for App {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer, 
        _: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {}
}

// Delegates

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_seat!(App);
delegate_layer!(App);
delegate_registry!(App);
delegate_pointer!(App);