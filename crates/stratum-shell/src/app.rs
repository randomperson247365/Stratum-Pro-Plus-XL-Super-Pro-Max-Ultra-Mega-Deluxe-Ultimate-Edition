use iced::widget::container;
use iced::{Element, Length, Subscription, Task, Theme};
use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::Application;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use stratum_ipc::IpcMessage;

use crate::ipc::ipc_subscription;
use crate::panel::panel_view;

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    /// Periodic tick for clock refresh.
    Tick,
    /// Incoming IPC event from stratum-wm.
    IpcEvent(IpcMessage),
}

/// iced_layershell requires Message to implement
/// TryInto<LayershellCustomActions, Error = Message>.
/// We never emit layershell actions, so we always return Err.
impl TryInto<LayershellCustomActions> for Message {
    type Error = Self;
    fn try_into(self) -> Result<LayershellCustomActions, Self> {
        Err(self)
    }
}

// ── Application state ─────────────────────────────────────────────────────────

pub struct ShellApp {
    pub focused_title: String,
}

impl Application for ShellApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                focused_title: String::new(),
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        "stratum-shell".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {} // clock widget reads Local::now() directly in view()
            Message::IpcEvent(IpcMessage::FocusChanged { title, .. }) => {
                self.focused_title = title;
            }
            Message::IpcEvent(_) => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        container(panel_view(self))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let ipc = ipc_subscription();
        let tick = iced::time::every(std::time::Duration::from_secs(30))
            .map(|_| Message::Tick);
        Subscription::batch([ipc, tick])
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
