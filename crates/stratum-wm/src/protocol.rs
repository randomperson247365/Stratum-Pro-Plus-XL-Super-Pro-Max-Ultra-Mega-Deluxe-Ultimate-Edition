//! Wayland protocol bindings for River, generated at compile time via wayland-scanner.
//!
//! Follows the same pattern as wayland-protocols: each protocol module has a
//! `pub mod __interfaces {}` submodule that holds the `generate_interfaces!` call,
//! then `use self::__interfaces::*` brings those statics into scope for
//! `generate_client_code!`. Cross-protocol references import the foreign
//! `__interfaces::*` inside the dependent module's own `__interfaces` submodule.

#![allow(
    dead_code,
    non_camel_case_types,
    unused_variables,
    unused_imports,
    clippy::all
)]

/// River window management protocol (the main WM interface).
pub mod river_window_management_v1 {
    use wayland_client::{self, protocol::*};
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "protocol/river-window-management-v1.xml"
        );
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocol/river-window-management-v1.xml");
}

/// XKB-based keybinding protocol.
pub mod river_xkb_bindings_v1 {
    use wayland_client::{self, protocol::*};
    // Bring the river_seat_v1 module into scope so generated client code
    // can reference river_seat_v1::RiverSeatV1.
    use super::river_window_management_v1::river_seat_v1;
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        // Bring river_seat_v1_interface (and friends) into scope so
        // generate_interfaces! can reference them for cross-protocol args.
        use super::super::river_window_management_v1::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "protocol/river-xkb-bindings-v1.xml"
        );
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocol/river-xkb-bindings-v1.xml");
}

/// Layer shell (used by stratum-shell in Phase 3).
pub mod river_layer_shell_v1 {
    use wayland_client::{self, protocol::*};
    use super::river_window_management_v1::river_seat_v1;
    use super::river_window_management_v1::river_output_v1;
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        use super::super::river_window_management_v1::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "protocol/river-layer-shell-v1.xml"
        );
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocol/river-layer-shell-v1.xml");
}

/// Input device management.
pub mod river_input_management_v1 {
    use wayland_client::{self, protocol::*};
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "protocol/river-input-management-v1.xml"
        );
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocol/river-input-management-v1.xml");
}

/// Libinput device configuration.
pub mod river_libinput_config_v1 {
    use wayland_client::{self, protocol::*};
    use super::river_input_management_v1::river_input_device_v1;
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        use super::super::river_input_management_v1::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "protocol/river-libinput-config-v1.xml"
        );
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocol/river-libinput-config-v1.xml");
}

// ── Convenience re-exports ───────────────────────────────────────────────────

pub use river_window_management_v1::river_seat_v1;
pub use river_window_management_v1::river_window_manager_v1::RiverWindowManagerV1;
pub use river_window_management_v1::river_window_v1::RiverWindowV1;
pub use river_seat_v1::RiverSeatV1;
pub use river_window_management_v1::river_output_v1::RiverOutputV1;
pub use river_window_management_v1::river_node_v1::RiverNodeV1;
pub use river_window_management_v1::river_decoration_v1::RiverDecorationV1;
pub use river_xkb_bindings_v1::river_xkb_bindings_v1::RiverXkbBindingsV1;
pub use river_xkb_bindings_v1::river_xkb_binding_v1::RiverXkbBindingV1;
pub use river_layer_shell_v1::river_layer_shell_v1::RiverLayerShellV1;
pub use river_input_management_v1::river_input_manager_v1::RiverInputManagerV1;
pub use river_libinput_config_v1::river_libinput_config_v1::RiverLibinputConfigV1;
