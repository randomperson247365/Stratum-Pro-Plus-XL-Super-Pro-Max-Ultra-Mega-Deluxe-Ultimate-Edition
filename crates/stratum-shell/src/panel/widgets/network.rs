use iced::widget::text;
use iced::Element;

use crate::app::Message;

/// Returns true if any physical (non-loopback) NIC has carrier.
fn has_network() -> bool {
    let Ok(dir) = std::fs::read_dir("/sys/class/net") else {
        return false;
    };
    for entry in dir.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "lo" {
            continue;
        }
        let carrier_path = entry.path().join("carrier");
        if let Ok(val) = std::fs::read_to_string(carrier_path) {
            if val.trim() == "1" {
                return true;
            }
        }
    }
    false
}

pub fn view<'a>() -> Element<'a, Message> {
    let label = if has_network() { " " } else { "󰖪 " };
    text(label).size(13).into()
}
