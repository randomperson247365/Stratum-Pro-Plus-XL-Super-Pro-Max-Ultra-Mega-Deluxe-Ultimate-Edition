use serde::{Deserialize, Serialize};

/// Information about a single open window, sent in WindowList responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app_id: String,
    pub title:  String,
}

/// All messages that can be exchanged over the IPC socket.
///
/// Uses serde's internally-tagged representation: `{ "type": "FocusChanged", "app_id": "...", ... }`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    /// Broadcast when a config value changes. `key` uses dot-path notation matching TOML paths.
    ConfigChanged {
        key:   String,
        value: serde_json::Value,
    },
    /// WM → shell: focused window changed.
    FocusChanged {
        app_id: String,
        title:  String,
    },
    /// WM → shell: active workspace changed.
    WorkspaceChanged {
        index: u32,
    },
    /// Launcher → WM: launch this command.
    SpawnApp {
        command: String,
    },
    /// Shell → WM: toggle tiling layout on the focused workspace.
    ToggleLayout,
    /// Keybind → shell: open the app launcher overlay.
    OpenLauncher,
    /// Any client → WM: request a list of open windows.
    GetWindowList,
    /// WM → client: response to GetWindowList.
    WindowList {
        windows: Vec<WindowInfo>,
    },
}
