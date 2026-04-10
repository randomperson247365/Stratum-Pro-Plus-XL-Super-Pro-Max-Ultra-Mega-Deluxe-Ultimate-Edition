use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Top-level config — single source of truth for all Stratum configuration.
/// All fields use #[serde(default)] so partial configs always work.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StratumConfig {
    pub general:      GeneralConfig,
    pub appearance:   AppearanceConfig,
    pub decorations:  DecorationsConfig,
    pub layout:       LayoutConfig,
    pub keybindings:  KeybindingsConfig,
    pub launcher:     LauncherConfig,
    pub panels:       Vec<PanelConfig>,
    pub window_rules: WindowRulesConfig,
    pub autostart:    AutostartConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub modifier: String,
    pub terminal: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            modifier: "super".into(),
            terminal: "foot".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme:        String,
    pub accent_color: String,
    pub font_ui:      String,
    pub font_mono:    String,
    pub gap_inner:    u32,
    pub gap_outer:    u32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme:        "dark".into(),
            accent_color: "#5e81f4".into(),
            font_ui:      "Inter".into(),
            font_mono:    "JetBrains Mono".into(),
            gap_inner:    8,
            gap_outer:    12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DecorationsConfig {
    pub titlebar_height:        u32,
    pub border_width_active:    u32,
    pub border_width_inactive:  u32,
    pub border_radius:          u32,
    pub shadow_enabled:         bool,
    pub shadow_spread:          u32,
    pub shadow_opacity:         f32,
    pub buttons_position:       String,
    pub buttons:                Vec<String>,
}

impl Default for DecorationsConfig {
    fn default() -> Self {
        Self {
            titlebar_height:       28,
            border_width_active:   4,
            border_width_inactive: 2,
            border_radius:         8,
            shadow_enabled:        true,
            shadow_spread:         12,
            shadow_opacity:        0.4,
            buttons_position:      "right".into(),
            buttons:               vec!["minimize".into(), "maximize".into(), "close".into()],
        }
    }
}

// ── Layout ───────────────────────────────────────────────────────────────────

/// Layout policy settings.
///
/// `default_mode` is one of `"floating"`, `"master_stack"`, or `"bsp"`.
///
/// `min_tile_width` / `min_tile_height` are the pixel thresholds (at 96 dpi) below
/// which the tiled layout automatically falls back to a deck arrangement.  When the
/// display reports its physical dimensions via `wl_output::Event::Geometry`, these
/// thresholds are scaled proportionally so a tile that is "too small to read" has
/// the same physical footprint regardless of pixel density.
///
/// `split_ratio` controls the BSP split proportion (0.5 = even halves).
/// Values outside [0.1, 0.9] are clamped at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub default_mode:    String,
    pub min_tile_width:  u32,
    pub min_tile_height: u32,
    pub split_ratio:     f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            default_mode:    "floating".into(),
            min_tile_width:  400,
            min_tile_height: 280,
            split_ratio:     0.5,
        }
    }
}

/// Maps "super+Return" → "spawn_terminal" etc.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KeybindingsConfig(pub HashMap<String, String>);

impl KeybindingsConfig {
    pub fn default_bindings() -> Self {
        let mut map = HashMap::new();
        map.insert("super+Return".into(), "spawn_terminal".into());
        map.insert("super+Space".into(),  "open_launcher".into());
        map.insert("super+q".into(),      "close_focused".into());
        map.insert("super+f".into(),      "toggle_fullscreen".into());
        map.insert("super+t".into(),      "toggle_tiling".into());
        map.insert("super+Tab".into(),    "focus_next".into());
        Self(map)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub show_recently_used: bool,
    pub show_categories:    bool,
    pub max_recent:         u32,
    pub search_settings:    bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            show_recently_used: true,
            show_categories:    true,
            max_recent:         8,
            search_settings:    true,
        }
    }
}

// ── Panel system ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PanelConfig {
    pub id:       String,
    pub screen:   u32,
    pub position: PanelPosition,
    pub height:   u32,
    pub autohide: bool,
    pub opacity:  f32,
    pub left:     Vec<WidgetConfig>,
    pub center:   Vec<WidgetConfig>,
    pub right:    Vec<WidgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PanelPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

/// One widget slot in a panel. Each variant carries its own optional config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetConfig {
    Workspaces,
    WindowTitle,
    Clock {
        #[serde(default = "default_clock_format")]
        format: String,
        #[serde(default = "default_true")]
        show_date: bool,
    },
    Tray,
    TrayFocused {
        pinned_app_id: String,
    },
    Battery {
        #[serde(default = "default_true")]
        show_percentage: bool,
    },
    Network,
    Media,
    QuickSettings,
}

fn default_clock_format() -> String { "%H:%M".into() }
fn default_true() -> bool { true }

// ── Window rules ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WindowRulesConfig {
    pub rules: Vec<WindowRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WindowRule {
    pub match_app_id: Vec<String>,
    pub floating:     bool,
    pub centered:     bool,
}

// ── Autostart ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AutostartConfig {
    pub programs: Vec<String>,
}
