pub mod widgets;

use iced::widget::{container, row, Space};
use iced::{Element, Length};

use crate::app::{Message, ShellApp};

/// Renders the full panel bar.
///
/// Layout: [window_title]  —spacer—  [clock]  [network]  [battery]
pub fn panel_view(app: &ShellApp) -> Element<'_, Message> {
    let left = widgets::window_title::view(&app.focused_title);

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
