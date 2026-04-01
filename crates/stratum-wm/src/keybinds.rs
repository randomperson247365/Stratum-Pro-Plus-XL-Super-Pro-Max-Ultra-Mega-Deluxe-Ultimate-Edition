use xkbcommon::xkb;

// river_seat_v1 modifier bitfield values (from river-window-management-v1.xml).
pub const MOD_SHIFT: u32 = 1;
pub const MOD_CTRL:  u32 = 4;
pub const MOD_ALT:   u32 = 8;   // mod1
pub const MOD_MOD3:  u32 = 32;
pub const MOD_SUPER: u32 = 64;  // mod4 / logo
pub const MOD_MOD5:  u32 = 128;

/// A parsed keybinding: xkbcommon keysym, River modifier bitflags, and the action name.
#[derive(Debug, Clone)]
pub struct ParsedKeybind {
    pub keysym:    xkb::Keysym,
    pub modifiers: u32,
    pub action:    String,
}

/// Parse a binding spec like "super+Return" or "super+shift+q".
///
/// The last `+`-separated token is the key name; everything before it is modifiers.
/// Returns None if the key name is unrecognised.
pub fn parse_keybind(spec: &str, action: &str) -> Option<ParsedKeybind> {
    let parts: Vec<&str> = spec.split('+').collect();
    if parts.is_empty() {
        return None;
    }
    let (mod_parts, key_slice) = parts.split_at(parts.len() - 1);
    let key_name = key_slice[0];

    // xkb::Keysym wraps a u32; NoSymbol = 0.
    let keysym = xkb::keysym_from_name(key_name, xkb::KEYSYM_NO_FLAGS);
    let keysym = if keysym.raw() == 0 {
        let ks2 = xkb::keysym_from_name(key_name, xkb::KEYSYM_CASE_INSENSITIVE);
        if ks2.raw() == 0 {
            eprintln!("stratum-wm: unknown key '{}' in binding '{}'", key_name, spec);
            return None;
        }
        ks2
    } else {
        keysym
    };

    Some(ParsedKeybind {
        keysym,
        modifiers: parse_modifiers(mod_parts),
        action:    action.to_string(),
    })
}

fn parse_modifiers(parts: &[&str]) -> u32 {
    let mut mods = 0u32;
    for part in parts {
        mods |= match part.to_lowercase().as_str() {
            "super" | "mod4" | "logo" => MOD_SUPER,
            "shift"                   => MOD_SHIFT,
            "ctrl" | "control"        => MOD_CTRL,
            "alt" | "mod1"            => MOD_ALT,
            "mod3"                    => MOD_MOD3,
            "mod5"                    => MOD_MOD5,
            other => {
                eprintln!("stratum-wm: unknown modifier '{}'", other);
                0
            }
        };
    }
    mods
}

/// Execute a WM action given a minimal context (avoids borrowing all of AppState).
pub fn execute_action(action: &str, ctx: ActionContext<'_>) {
    match action {
        "spawn_terminal" => spawn_process(ctx.terminal),
        "close_focused" => {
            if let Some(win) = ctx.focused_window {
                win.close();
            }
        }
        "toggle_fullscreen" | "focus_next" | "open_launcher" | "toggle_tiling" => {
            // These are handled via the manage sequence in state.rs.
        }
        other => {
            if let Some(cmd) = other.strip_prefix("spawn:") {
                spawn_process(cmd);
            } else {
                eprintln!("stratum-wm: unknown action '{}'", other);
            }
        }
    }
}

/// Minimal context for execute_action.
pub struct ActionContext<'a> {
    pub terminal: &'a str,
    pub focused_window: Option<&'a crate::protocol::RiverWindowV1>,
}

pub fn spawn_process(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match std::process::Command::new(parts[0]).args(&parts[1..]).spawn() {
        Ok(_) => {}
        Err(e) => eprintln!("stratum-wm: failed to spawn '{}': {}", cmd, e),
    }
}
