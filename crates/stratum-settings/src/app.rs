use std::path::PathBuf;

use iced::widget::{column, container, horizontal_rule, row, text};
use iced::{Element, Length, Task};
use stratum_config::{default_config_path, load_config, save_config, StratumConfig};

use crate::tabs::{self, Tab};

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    // Appearance
    AccentColorChanged(String),
    GapInnerChanged(String),
    GapOuterChanged(String),
    ThemeToggled(bool),
    FontUiChanged(String),
    FontMonoChanged(String),
    // Decorations
    TitlebarHeightChanged(String),
    BorderActiveChanged(String),
    BorderInactiveChanged(String),
    BorderRadiusChanged(String),
    ShadowToggled(bool),
    ShadowSpreadChanged(String),
    ShadowOpacityChanged(String),
    // Keybindings
    KeyChanged(usize, String),
    ActionChanged(usize, String),
    AddKeybind,
    RemoveKeybind(usize),
    // Persist
    Save,
    Reset,
}

// ── Edit state ────────────────────────────────────────────────────────────────
//
// All numeric config values are mirrored as String fields so that text_input
// widgets can borrow them without lifetime issues (text_input takes &str).
// They are parsed back to numeric types only on Save.

pub struct SettingsApp {
    config_path: PathBuf,
    active_tab:  Tab,
    status:      String,

    // Appearance
    pub accent_color: String,
    pub dark_mode:    bool,
    pub gap_inner:    String,
    pub gap_outer:    String,
    pub font_ui:      String,
    pub font_mono:    String,

    // Decorations
    pub titlebar_height:  String,
    pub border_active:    String,
    pub border_inactive:  String,
    pub border_radius:    String,
    pub shadow_enabled:   bool,
    pub shadow_spread:    String,
    pub shadow_opacity:   String,

    // Keybindings
    pub keybinds: Vec<(String, String)>,
}

impl SettingsApp {
    fn from_config(config: &StratumConfig, config_path: PathBuf) -> Self {
        let mut keybinds: Vec<(String, String)> = config
            .keybindings
            .0
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        keybinds.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            config_path,
            active_tab: Tab::Appearance,
            status: String::new(),

            accent_color: config.appearance.accent_color.clone(),
            dark_mode:    config.appearance.theme == "dark",
            gap_inner:    config.appearance.gap_inner.to_string(),
            gap_outer:    config.appearance.gap_outer.to_string(),
            font_ui:      config.appearance.font_ui.clone(),
            font_mono:    config.appearance.font_mono.clone(),

            titlebar_height: config.decorations.titlebar_height.to_string(),
            border_active:   config.decorations.border_width_active.to_string(),
            border_inactive: config.decorations.border_width_inactive.to_string(),
            border_radius:   config.decorations.border_radius.to_string(),
            shadow_enabled:  config.decorations.shadow_enabled,
            shadow_spread:   config.decorations.shadow_spread.to_string(),
            shadow_opacity:  format!("{:.2}", config.decorations.shadow_opacity),

            keybinds,
        }
    }

    /// Reconstitute a StratumConfig from the current edit state, then save it.
    fn build_and_save(&self) -> Result<(), String> {
        let mut config = load_config(&self.config_path).unwrap_or_default();

        config.appearance.accent_color = self.accent_color.clone();
        config.appearance.theme = if self.dark_mode { "dark" } else { "light" }.into();
        if let Ok(v) = self.gap_inner.trim().parse() { config.appearance.gap_inner = v; }
        if let Ok(v) = self.gap_outer.trim().parse() { config.appearance.gap_outer = v; }
        config.appearance.font_ui   = self.font_ui.clone();
        config.appearance.font_mono = self.font_mono.clone();

        if let Ok(v) = self.titlebar_height.trim().parse() { config.decorations.titlebar_height = v; }
        if let Ok(v) = self.border_active.trim().parse()   { config.decorations.border_width_active = v; }
        if let Ok(v) = self.border_inactive.trim().parse() { config.decorations.border_width_inactive = v; }
        if let Ok(v) = self.border_radius.trim().parse()   { config.decorations.border_radius = v; }
        config.decorations.shadow_enabled = self.shadow_enabled;
        if let Ok(v) = self.shadow_spread.trim().parse()  { config.decorations.shadow_spread = v; }
        if let Ok(v) = self.shadow_opacity.trim().parse::<f32>() {
            config.decorations.shadow_opacity = v.clamp(0.0, 1.0);
        }

        config.keybindings.0 = self.keybinds
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        save_config(&config, &self.config_path).map_err(|e| e.to_string())
    }
}

// ── Init / update / view ──────────────────────────────────────────────────────

pub fn init() -> (SettingsApp, Task<Message>) {
    let config_path = default_config_path();
    let config = load_config(&config_path).unwrap_or_default();
    (SettingsApp::from_config(&config, config_path), Task::none())
}

pub fn update(state: &mut SettingsApp, msg: Message) -> Task<Message> {
    match msg {
        Message::TabSelected(tab) => state.active_tab = tab,

        Message::AccentColorChanged(v)   => state.accent_color = v,
        Message::GapInnerChanged(v)      => state.gap_inner    = v,
        Message::GapOuterChanged(v)      => state.gap_outer    = v,
        Message::ThemeToggled(dark)      => state.dark_mode    = dark,
        Message::FontUiChanged(v)        => state.font_ui      = v,
        Message::FontMonoChanged(v)      => state.font_mono    = v,

        Message::TitlebarHeightChanged(v) => state.titlebar_height = v,
        Message::BorderActiveChanged(v)   => state.border_active   = v,
        Message::BorderInactiveChanged(v) => state.border_inactive = v,
        Message::BorderRadiusChanged(v)   => state.border_radius   = v,
        Message::ShadowToggled(v)         => state.shadow_enabled  = v,
        Message::ShadowSpreadChanged(v)   => state.shadow_spread   = v,
        Message::ShadowOpacityChanged(v)  => state.shadow_opacity  = v,

        Message::KeyChanged(i, v)    => { if let Some(r) = state.keybinds.get_mut(i) { r.0 = v; } }
        Message::ActionChanged(i, v) => { if let Some(r) = state.keybinds.get_mut(i) { r.1 = v; } }
        Message::AddKeybind          => state.keybinds.push((String::new(), String::new())),
        Message::RemoveKeybind(i)    => { if i < state.keybinds.len() { state.keybinds.remove(i); } }

        Message::Save => {
            match state.build_and_save() {
                Ok(())  => state.status = format!("Saved to {}", state.config_path.display()),
                Err(e)  => state.status = format!("Error: {e}"),
            }
        }
        Message::Reset => {
            let config = load_config(&state.config_path).unwrap_or_default();
            let path = state.config_path.clone();
            *state = SettingsApp::from_config(&config, path);
            state.status = "Reset to saved values.".into();
        }
    }
    Task::none()
}

pub fn view(state: &SettingsApp) -> Element<'_, Message> {
    let tab_bar = tabs::tab_bar(&state.active_tab);

    let content: Element<Message> = match state.active_tab {
        Tab::Appearance  => tabs::appearance::view(state),
        Tab::Decorations => tabs::decorations::view(state),
        Tab::Keybindings => tabs::keybindings::view(&state.keybinds),
    };

    let status_bar = row![
        text(&state.status).size(12),
        iced::widget::Space::with_width(Length::Fill),
        iced::widget::button("Reset").on_press(Message::Reset),
        iced::widget::button("Save").on_press(Message::Save),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let layout = column![
        tab_bar,
        horizontal_rule(1),
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16),
        horizontal_rule(1),
        container(status_bar)
            .width(Length::Fill)
            .padding([8, 16]),
    ]
    .spacing(0);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
