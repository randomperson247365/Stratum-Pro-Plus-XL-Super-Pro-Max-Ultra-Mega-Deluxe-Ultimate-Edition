mod app;
mod ipc;
mod launcher;
mod panel;

use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};

/// Quick probe: check that the Wayland compositor advertises zwlr_layer_shell_v1.
/// layershellev panics (instead of returning an error) if the global is missing,
/// so we must check before attempting to start the iced_layershell event loop.
struct ProtocolProbe {
    has_layer_shell: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for ProtocolProbe {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {
            if interface == "zwlr_layer_shell_v1" {
                state.has_layer_shell = true;
            }
        }
    }
}

fn compositor_has_layer_shell() -> bool {
    let Ok(conn) = Connection::connect_to_env() else { return false };
    let mut eq = conn.new_event_queue::<ProtocolProbe>();
    let qh = eq.handle();
    conn.display().get_registry(&qh, ());
    let mut state = ProtocolProbe { has_layer_shell: false };
    let _ = eq.roundtrip(&mut state);
    state.has_layer_shell
}

fn main() -> anyhow::Result<()> {
    if !compositor_has_layer_shell() {
        eprintln!(
            "stratum-shell: zwlr_layer_shell_v1 not available — \
             panel requires a compositor that supports the wlr-layer-shell protocol \
             (e.g. River, Sway, Hyprland).  Exiting."
        );
        return Ok(());
    }
    app::run()
}
