use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::{Color, Element, Length};

use crate::app::{Message, ShellApp};
use crate::launcher::AppEntry;

const OVERLAY_BG:  Color = Color { r: 0.05, g: 0.05, b: 0.08, a: 0.92 };
const ROW_BG:      Color = Color { r: 0.12, g: 0.12, b: 0.16, a: 1.0 };
const ROW_HOVER:   Color = Color { r: 0.20, g: 0.20, b: 0.28, a: 1.0 };

pub fn launcher_view(app: &ShellApp) -> Element<'_, Message> {
    let search_input = text_input("Type to search apps...", &app.launcher_query)
        .id(text_input::Id::new("launcher-search"))
        .on_input(Message::QueryChanged)
        .padding(12)
        .size(18)
        .width(Length::Fixed(480.0));

    let results: Element<Message> = if app.filtered_apps.is_empty() {
        text("No results").size(14).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    } else {
        let rows = app
            .filtered_apps
            .iter()
            .map(|entry| app_row(entry))
            .collect::<Vec<_>>();

        scrollable(
            column(rows)
                .spacing(2)
                .width(Length::Fixed(480.0)),
        )
        .height(Length::Fixed(320.0))
        .into()
    };

    let card = column![search_input, results]
        .spacing(12)
        .padding(24);

    // Full-screen overlay with centred card.
    container(
        container(card)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ROW_BG)),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(OVERLAY_BG)),
        ..Default::default()
    })
    .into()
}

fn app_row(entry: &AppEntry) -> Element<'_, Message> {
    button(
        text(entry.name.as_str()).size(15),
    )
    .on_press(Message::Launch(entry.exec.clone()))
    .width(Length::Fill)
    .padding([10, 16])
    .style(|_, status| {
        let bg = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            ROW_HOVER
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
