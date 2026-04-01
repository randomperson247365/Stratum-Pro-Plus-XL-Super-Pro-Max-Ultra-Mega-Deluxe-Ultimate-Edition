use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

use crate::load_config;
use crate::schema::StratumConfig;

/// Spawns a background thread watching `path` for changes.
/// Returns a Receiver that yields a new StratumConfig each time the file changes.
pub fn watch_config(path: PathBuf) -> mpsc::Receiver<StratumConfig> {
    let (tx, rx) = mpsc::channel();
    let watch_path = path.clone();

    std::thread::spawn(move || {
        let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match recommended_watcher(notify_tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("stratum-config: failed to create file watcher: {e}");
                return;
            }
        };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
            eprintln!("stratum-config: failed to watch {}: {e}", watch_path.display());
            return;
        }

        for result in notify_rx {
            match result {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_)
                    ) {
                        // Debounce: wait a moment for the write to complete
                        std::thread::sleep(Duration::from_millis(50));
                        match load_config(&watch_path) {
                            Ok(config) => {
                                let _ = tx.send(config);
                            }
                            Err(e) => {
                                eprintln!("stratum-config: failed to reload config: {e}");
                            }
                        }
                    }
                }
                Err(e) => eprintln!("stratum-config: watcher error: {e}"),
            }
        }
    });

    rx
}
