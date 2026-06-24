use std::io::Read;
use std::os::fd::{AsFd, FromRawFd, OwnedFd};

use crate::app::App;
use crate::utils::{nix_pipe, percent_decode, output};

use wayland_client::{
    protocol::{
        wl_data_device::{self, WlDataDevice},
        wl_data_device_manager::DndAction,
        wl_data_offer::{self, WlDataOffer},
    },
    Connection, Dispatch, QueueHandle,
};

impl Dispatch<WlDataDevice, ()> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlDataDevice,
        event: wl_data_device::Event,
        _data: &(),
        conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => {
                state.pending_offer = Some(id);
                state.offer_has_uri = false;
            }

            wl_data_device::Event::Enter { serial, id, .. } => {
                if let Some(offer) = &id {
                    offer.accept(serial, Some("text/uri-list".to_string()));
                    // Fixed: Using the strict DndAction enum instead of integers
                    offer.set_actions(DndAction::Copy | DndAction::Move, DndAction::Copy);
                }
            }

            wl_data_device::Event::Motion { .. } => {
                if let Some(offer) = &state.pending_offer {
                    offer.accept(0, Some("text/uri-list".to_string()));
                }
            }

            wl_data_device::Event::Drop => {
                if state.offer_has_uri {
                    if let Some(offer) = state.pending_offer.take() {
                        let (read_fd, write_fd) = nix_pipe();
                        
                        // Fixed: Convert raw i32 to a safe OwnedFd, then borrow it for Wayland
                        let write_owned = unsafe { OwnedFd::from_raw_fd(write_fd) };
                        offer.receive("text/uri-list".to_string(), write_owned.as_fd());
                        drop(write_owned);
                        
                        let _ = conn.flush();

                        let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
                        let mut buf = String::new();
                        let _ = file.read_to_string(&mut buf);

                        for line in buf.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
                                continue;
                            }
                            let path = if let Some(p) = line.strip_prefix("file://") {
                                let plain_p = p.strip_prefix("localhost").unwrap_or(p);
                                percent_decode(plain_p)
                            } else {
                                line.to_string()
                            };
                            output(path);
                        }

                        offer.finish();
                        offer.destroy();
                        state.offer_has_uri = false;
                    }
                }else {
                    if let Some(offer) = state.pending_offer.take() {
                        offer.destroy();
                    }
                }
            }

            wl_data_device::Event::Leave => {
                if let Some(offer) = state.pending_offer.take() {
                    offer.destroy();
                }
                state.offer_has_uri = false;
            }

            _ => {}
        }
    }
}

impl Dispatch<WlDataOffer, ()> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlDataOffer,
        event: wl_data_offer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event {
            if mime_type == "text/uri-list" {
                state.offer_has_uri = true;
            }
        }
    }
}