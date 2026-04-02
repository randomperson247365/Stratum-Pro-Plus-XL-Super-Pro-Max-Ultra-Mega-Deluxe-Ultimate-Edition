mod app;
mod ipc;
mod launcher;
mod panel;

fn main() -> anyhow::Result<()> {
    app::run()
}
