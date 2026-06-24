use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerSurface,
    shm::{slot::SlotPool, Shm},
};
use wayland_client::protocol::{
    wl_data_device::WlDataDevice,
    wl_data_device_manager::WlDataDeviceManager,
    wl_data_offer::WlDataOffer,
};

pub struct App {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub seat_state: SeatState,
    pub layer_surface: LayerSurface,
    pub pool: Option<SlotPool>,
    pub configured: bool,
    pub data_device_manager: Option<WlDataDeviceManager>,
    pub data_device: Option<WlDataDevice>,
    pub pending_offer: Option<WlDataOffer>,
    pub offer_has_uri: bool,
}

impl App {
    pub fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        shm: Shm,
        seat_state: SeatState,
        layer_surface: LayerSurface,
        data_device_manager: Option<WlDataDeviceManager>,
    ) -> Self {
        Self {
            registry_state,
            output_state,
            shm,
            seat_state,
            layer_surface,
            pool: None,
            configured: false,
            data_device_manager,
            data_device: None,
            pending_offer: None,
            offer_has_uri: false,
        }
    }
}