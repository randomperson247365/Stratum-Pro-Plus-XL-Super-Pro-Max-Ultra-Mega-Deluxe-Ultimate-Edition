use iced::widget::text;
use iced::Element;

use crate::app::Message;

pub fn view<'a>(title: &'a str) -> Element<'a, Message> {
    text(if title.is_empty() { "Desktop" } else { title })
        .size(13)
        .into()
}
