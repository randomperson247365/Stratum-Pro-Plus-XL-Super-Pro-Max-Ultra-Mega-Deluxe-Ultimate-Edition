use iced::widget::{checkbox, column, pick_list, row, text, text_input};
use iced::{Element, Length};

use crate::app::{Message, SettingsApp};

const LAYOUT_MODES: &[&str] = &["floating", "master_stack", "bsp"];

pub fn view(state: &SettingsApp) -> Element<'_, Message> {
    let layout_pick = row![
        text("Default layout mode").width(Length::Fixed(200.0)),
        pick_list(
            LAYOUT_MODES,
            Some(state.layout_mode.as_str()),
            |s: &str| Message::LayoutModeChanged(s.to_owned()),
        )
        .width(Length::Fixed(160.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    column![
        section_label("Theme"),
        checkbox("Dark mode", state.dark_mode)
            .on_toggle(Message::ThemeToggled),

        section_label("Accent color"),
        text_input("#rrggbb", &state.accent_color)
            .on_input(Message::AccentColorChanged)
            .width(160),

        section_label("Window gaps"),
        labeled_input("Inner gap (px)", &state.gap_inner, Message::GapInnerChanged),
        labeled_input("Outer gap (px)", &state.gap_outer, Message::GapOuterChanged),

        section_label("Layout"),
        layout_pick,
        labeled_input("Min tile width (px @ 96dpi)",  &state.min_tile_width,  Message::MinTileWidthChanged),
        labeled_input("Min tile height (px @ 96dpi)", &state.min_tile_height, Message::MinTileHeightChanged),

        section_label("Fonts"),
        labeled_input("UI font",   &state.font_ui,   Message::FontUiChanged),
        labeled_input("Mono font", &state.font_mono, Message::FontMonoChanged),
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
        text(label).width(Length::Fixed(160.0)),
        text_input("", value)
            .on_input(on_input)
            .width(Length::Fixed(200.0)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
