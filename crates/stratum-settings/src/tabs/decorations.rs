use iced::widget::{checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::app::{Message, SettingsApp};

pub fn view(state: &SettingsApp) -> Element<'_, Message> {
    column![
        section_label("Titlebar"),
        labeled_input("Height (px)",      &state.titlebar_height, Message::TitlebarHeightChanged),
        labeled_input("Border radius (px)", &state.border_radius,  Message::BorderRadiusChanged),

        section_label("Borders"),
        labeled_input("Active width (px)",   &state.border_active,   Message::BorderActiveChanged),
        labeled_input("Inactive width (px)", &state.border_inactive, Message::BorderInactiveChanged),

        section_label("Shadow"),
        checkbox("Enable shadow", state.shadow_enabled)
            .on_toggle(Message::ShadowToggled),
        labeled_input("Spread (px)",       &state.shadow_spread,  Message::ShadowSpreadChanged),
        labeled_input("Opacity (0–1)",     &state.shadow_opacity, Message::ShadowOpacityChanged),
    ]
    .spacing(8)
    .width(Length::Fill)
    .into()
}

fn section_label(label: &str) -> iced::widget::Text<'_> {
    text(label).size(13).color(iced::Color::from_rgb(0.5, 0.5, 0.5))
}

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(180.0)),
        text_input("", value)
            .on_input(on_input)
            .width(Length::Fixed(160.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
