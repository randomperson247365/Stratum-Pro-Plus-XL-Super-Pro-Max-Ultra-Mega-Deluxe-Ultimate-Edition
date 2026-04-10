use iced::widget::text;
use iced::Element;

use crate::app::Message;

struct BatteryInfo {
    capacity: u8,
    charging: bool,
}

/// Scans /sys/class/power_supply/ for the first BAT* entry.
fn read_battery() -> Option<BatteryInfo> {
    let dir = std::fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("BAT") {
            continue;
        }
        let base = entry.path();
        let capacity = std::fs::read_to_string(base.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())?;
        let charging = std::fs::read_to_string(base.join("status"))
            .map(|s| s.trim() == "Charging")
            .unwrap_or(false);
        return Some(BatteryInfo { capacity, charging });
    }
    None
}

pub fn view<'a>() -> Element<'a, Message> {
    let label = match read_battery() {
        Some(b) => {
            let icon = if b.charging {
                "󰂄"
            } else if b.capacity > 80 {
                ""
            } else if b.capacity > 50 {
                ""
            } else if b.capacity > 20 {
                ""
            } else {
                ""
            };
            format!("{icon} {}%", b.capacity)
        }
        None => String::new(), // no battery — show nothing
    };
    text(label).size(13).into()
}
