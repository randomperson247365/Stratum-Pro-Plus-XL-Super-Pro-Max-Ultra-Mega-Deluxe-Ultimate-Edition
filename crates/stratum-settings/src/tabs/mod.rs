pub mod appearance;
pub mod decorations;
pub mod keybindings;

use iced::widget::{button, row};
use iced::{Element, Length};

use crate::app::Message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Appearance,
    Decorations,
    Keybindings,
}

pub fn tab_bar(active: &Tab) -> Element<'static, Message> {
    let tabs = [
        ("Appearance",  Tab::Appearance),
        ("Decorations", Tab::Decorations),
        ("Keybindings", Tab::Keybindings),
    ];

    let buttons = tabs.into_iter().map(|(label, tab)| {
        let is_active = *active == tab;
        button(label)
            .on_press(Message::TabSelected(tab))
            .style(move |theme: &iced::Theme, status| {
                let pair = theme.extended_palette().primary;
                if is_active {
                    button::Style {
                        background: Some(iced::Background::Color(pair.strong.color)),
                        text_color: pair.strong.text,
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                } else {
                    let bg = match status {
                        button::Status::Hovered => pair.weak.color,
                        _ => iced::Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: theme.extended_palette().background.base.text,
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                }
            })
            .into()
    });

    row(buttons)
        .spacing(4)
        .width(Length::Fill)
        .into()
}
