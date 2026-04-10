use chrono::Local;
use iced::widget::text;
use iced::Element;

use crate::app::Message;

pub fn view<'a>() -> Element<'a, Message> {
    let now = Local::now();
    // Format includes seconds so the 1-second tick keeps display accurate.
    text(now.format("%a %b %-d  %H:%M:%S").to_string())
        .size(13)
        .into()
}
