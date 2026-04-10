pub mod widgets;

use iced::widget::{button, container, row, text, Space};
use iced::{Color, Element, Length};

use crate::app::{Message, ShellApp};

/// Renders the full panel bar.
///
/// Layout: [⊞ launcher btn]  [window_title]  —spacer—  [clock]  [network]  [battery]
pub fn panel_view(app: &ShellApp) -> Element<'_, Message> {
    let launcher_btn = button(
        text("⊞").size(16).color(Color::WHITE),
    )
    .on_press(Message::OpenLauncher)
    .padding(iced::Padding::from([4, 10]))
    .style(|_, status| button::Style {
        background: Some(iced::Background::Color(
            if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                Color { r: 0.3, g: 0.3, b: 0.4, a: 1.0 }
            } else {
                Color::TRANSPARENT
            }
        )),
        text_color: Color::WHITE,
        border: iced::Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    });

    let left = row![
        launcher_btn,
        Space::with_width(4),
        widgets::window_title::view(&app.focused_title),
    ]
    .align_y(iced::Alignment::Center);

    let center_right = row![
        widgets::clock::view(),
        Space::with_width(12),
        widgets::network::view(),
        Space::with_width(6),
        widgets::battery::view(),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let content = row![
        container(left)
            .padding(iced::Padding::from([0, 8]))
            .align_y(iced::alignment::Vertical::Center),
        Space::with_width(Length::Fill),
        container(center_right)
            .padding(iced::Padding::from([0, 8]))
            .align_y(iced::alignment::Vertical::Center),
    ]
    .height(Length::Fill)
    .align_y(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
