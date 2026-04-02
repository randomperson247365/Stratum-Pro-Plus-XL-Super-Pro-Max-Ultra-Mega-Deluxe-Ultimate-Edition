mod app;
mod tabs;

fn main() -> iced::Result {
    iced::application("Stratum Settings", app::update, app::view)
        .window_size((860.0, 560.0))
        .run_with(app::init)
}
