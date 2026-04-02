use chrono::Local;
use iced::widget::text;
use iced::Element;

use crate::app::Message;

pub fn view<'a>() -> Element<'a, Message> {
    let now = Local::now();
    text(now.format("%a %b %-d  %H:%M").to_string())
        .size(13)
        .into()
}
