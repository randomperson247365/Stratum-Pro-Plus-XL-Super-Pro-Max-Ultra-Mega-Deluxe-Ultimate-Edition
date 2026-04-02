use iced::widget::text;
use iced::Element;

use crate::app::Message;

/// Reads /sys/class/power_supply/BAT0/capacity (0-100) and returns a label.
/// Returns None if no battery is present.
fn read_capacity() -> Option<u8> {
    std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
}

pub fn view<'a>() -> Element<'a, Message> {
    let label = match read_capacity() {
        Some(pct) => {
            let icon = if pct > 80 {
                ""
            } else if pct > 50 {
                ""
            } else if pct > 20 {
                ""
            } else {
                ""
            };
            format!("{icon} {pct}%")
        }
        None => String::new(), // no battery — show nothing
    };
    text(label).size(13).into()
}
