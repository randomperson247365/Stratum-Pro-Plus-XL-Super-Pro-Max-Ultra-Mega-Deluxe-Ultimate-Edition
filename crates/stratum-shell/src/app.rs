use iced::widget::container;
use iced::{Element, Length, Subscription, Task, Theme};
use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::Application;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use stratum_ipc::IpcMessage;

use crate::ipc::ipc_subscription;
use crate::launcher::{self, AppEntry};
use crate::launcher::view::launcher_view;
use crate::panel::panel_view;

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    /// Periodic tick for clock refresh.
    Tick,
    /// Incoming IPC event from stratum-wm.
    IpcEvent(IpcMessage),
    // Launcher
    QueryChanged(String),
    Launch(String),   // exec string
    CloseLauncher,
    // Layershell geometry commands (routed via TryInto, never reach update()).
    GoFullscreen,
    GoPanel,
}

/// iced_layershell routes messages through TryInto<LayershellCustomActions>
/// before delivering them to update(). Variants that convert successfully are
/// consumed as layer-shell actions; the rest (Err) proceed to update() as normal.
impl TryInto<LayershellCustomActions> for Message {
    type Error = Self;

    fn try_into(self) -> Result<LayershellCustomActions, Self> {
        match self {
            // Expand the window to fill the whole screen.
            Message::GoFullscreen => Ok(LayershellCustomActions::AnchorSizeChange(
                Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom,
                (0, 0),
            )),
            // Shrink back to a 40 px bottom strip.
            Message::GoPanel => Ok(LayershellCustomActions::AnchorSizeChange(
                Anchor::Left | Anchor::Right | Anchor::Bottom,
                (0, 40),
            )),
            other => Err(other),
        }
    }
}

// ── Application state ─────────────────────────────────────────────────────────

pub struct ShellApp {
    pub focused_title:  String,
    pub launcher_open:  bool,
    pub launcher_query: String,
    pub all_apps:       Vec<AppEntry>,
    pub filtered_apps:  Vec<AppEntry>,
}

impl Application for ShellApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<Message>) {
        let all_apps = launcher::load_apps();
        let filtered_apps = all_apps.iter().take(8).cloned().collect();
        (
            Self {
                focused_title: String::new(),
                launcher_open: false,
                launcher_query: String::new(),
                all_apps,
                filtered_apps,
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        "stratum-shell".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => Task::none(),

            Message::IpcEvent(msg) => match msg {
                IpcMessage::FocusChanged { title, .. } => {
                    self.focused_title = title;
                    Task::none()
                }
                IpcMessage::OpenLauncher => {
                    self.launcher_open = true;
                    self.launcher_query.clear();
                    self.filtered_apps = self.all_apps.iter().take(8).cloned().collect();
                    // Resize to fullscreen — routed through TryInto, not update().
                    Task::done(Message::GoFullscreen)
                }
                _ => Task::none(),
            },

            Message::QueryChanged(q) => {
                self.filtered_apps = launcher::fuzzy_filter(&self.all_apps, &q)
                    .into_iter()
                    .cloned()
                    .collect();
                self.launcher_query = q;
                Task::none()
            }

            Message::Launch(exec) => {
                let _ = std::process::Command::new("sh")
                    .args(["-c", &exec])
                    .spawn();
                self.launcher_open = false;
                self.launcher_query.clear();
                Task::done(Message::GoPanel)
            }

            Message::CloseLauncher => {
                self.launcher_open = false;
                self.launcher_query.clear();
                Task::done(Message::GoPanel)
            }

            // These are intercepted by TryInto before reaching update().
            Message::GoFullscreen | Message::GoPanel => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        if self.launcher_open {
            launcher_view(self)
        } else {
            container(panel_view(self))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let ipc = ipc_subscription();
        let tick = iced::time::every(std::time::Duration::from_secs(30))
            .map(|_| Message::Tick);

        // Close launcher on Escape.
        let esc = iced::keyboard::on_key_press(|key, _mods| {
            if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                Some(Message::CloseLauncher)
            } else {
                None
            }
        });

        Subscription::batch([ipc, tick, esc])
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> anyhow::Result<()> {
    ShellApp::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((0, 40)),
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
            exclusive_zone: 40,
            layer: Layer::Top,
            ..Default::default()
        },
        ..Settings::default()
    })?;
    Ok(())
}
