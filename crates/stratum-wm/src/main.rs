mod decorations;
mod keybinds;
mod layout;
mod output;
mod protocol;
mod seat;
mod state;
mod window;

use stratum_config::{default_config_path, load_config};
use stratum_ipc::IpcServer;
use wayland_client::Connection;

use state::AppState;

fn main() -> anyhow::Result<()> {
    // Load config (falls back to defaults if absent or unparseable).
    let config_path = default_config_path();
    let config = load_config(&config_path).unwrap_or_default();

    // Start the inotify config watcher in a background thread.
    let config_rx = stratum_config::watch_config(config_path);

    // Start a tokio multi-thread runtime for the IPC server.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("stratum-wm: failed to build tokio runtime");

    // Bind the IPC socket and get a broadcast sender.
    let ipc_tx = match IpcServer::bind() {
        Ok(server) => {
            let tx = server.tx.clone();
            rt.spawn(server.run());
            Some(tx)
        }
        Err(e) => {
            eprintln!("stratum-wm: IPC server bind failed: {e} — shell features disabled");
            None
        }
    };

    // Connect to the Wayland display (reads $WAYLAND_DISPLAY).
    let conn = Connection::connect_to_env()
        .expect("stratum-wm: failed to connect to Wayland — is River running?");

    let mut event_queue = conn.new_event_queue::<AppState>();
    let qh = event_queue.handle();

    // Register for global advertisement.
    conn.display().get_registry(&qh, ());

    let mut state = AppState::new(config);

    if let Some(tx) = ipc_tx {
        state.set_ipc_tx(tx);
    }

    // First roundtrip: receive all globals.
    event_queue.roundtrip(&mut state)?;
    // Second roundtrip: River sends initial manage_start, window/output/seat events.
    event_queue.roundtrip(&mut state)?;

    // Register keybindings for any seats already received.
    state.register_keybinds(&qh);

    eprintln!("stratum-wm: connected to River");

    // Main loop: block for the next event, then check config watcher.
    while state.running {
        // Block until a Wayland event arrives, then dispatch all pending ones.
        event_queue.blocking_dispatch(&mut state)?;

        // Check for a hot-reloaded config (non-blocking).
        if let Ok(new_config) = config_rx.try_recv() {
            eprintln!("stratum-wm: config reloaded — re-registering keybinds");
            state.config = new_config;
            state.register_keybinds(&qh);
        }
    }

    eprintln!("stratum-wm: exiting");
    Ok(())
}
