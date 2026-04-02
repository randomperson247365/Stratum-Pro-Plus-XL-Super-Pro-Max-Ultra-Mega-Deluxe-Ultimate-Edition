use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::app::Message;

pub fn view(binds: &[(String, String)]) -> Element<'_, Message> {
    let header = row![
        text("Key binding").width(Length::Fixed(220.0)).size(13)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text("Action").width(Length::Fill).size(13)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text("").width(Length::Fixed(32.0)),
    ]
    .spacing(8);

    let rows: Vec<Element<Message>> = binds
        .iter()
        .enumerate()
        .map(|(i, (key, action))| {
            row![
                text_input("super+key", key)
                    .on_input(move |s| Message::KeyChanged(i, s))
                    .width(Length::Fixed(220.0)),
                text_input("action", action)
                    .on_input(move |s| Message::ActionChanged(i, s))
                    .width(Length::Fill),
                button("×")
                    .on_press(Message::RemoveKeybind(i))
                    .width(Length::Fixed(32.0))
                    .style(|_, _| button::Style {
                        background: Some(iced::Background::Color(
                            iced::Color::from_rgb(0.6, 0.1, 0.1),
                        )),
                        text_color: iced::Color::WHITE,
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into()
        })
        .collect();

    let list = scrollable(
        column(rows).spacing(4),
    )
    .height(Length::Fill);

    let add_btn = button("+ Add binding")
        .on_press(Message::AddKeybind);

    column![header, list, add_btn]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
