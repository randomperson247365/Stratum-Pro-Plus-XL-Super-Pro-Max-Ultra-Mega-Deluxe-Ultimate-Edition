use crate::protocol::{RiverSeatV1, RiverXkbBindingV1};

/// A keybinding registered with River: the protocol object and action string.
pub struct RegisteredKeybind {
    pub binding: RiverXkbBindingV1,
    pub action:  String,
}

/// State tracked for each seat (a keyboard + pointer pair).
pub struct SeatState {
    pub proxy:              RiverSeatV1,
    pub registered_binds:   Vec<RegisteredKeybind>,
    /// Global name of the correlated wl_seat.
    pub wl_seat_name:       Option<u32>,
}

impl SeatState {
    pub fn new(proxy: RiverSeatV1) -> Self {
        Self {
            proxy,
            registered_binds: Vec::new(),
            wl_seat_name:     None,
        }
    }

    /// Disable and destroy all registered keybindings (before re-registering on config reload).
    pub fn clear_keybinds(&mut self) {
        for bind in self.registered_binds.drain(..) {
            bind.binding.disable();
            bind.binding.destroy();
        }
    }
}
